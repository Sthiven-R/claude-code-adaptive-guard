"""
transcript.py - Read Claude Code session transcripts safely.

The transcript is a JSONL file that Claude Code writes to
`~/.claude/projects/<project-hash>/<session-id>.jsonl`. Each line is a
turn event. This module reads the tail of the file and extracts:

  - The most recent user prompt (as text).
  - The most recent assistant response (as visible text — thinking and
    tool_use blocks are NOT counted as visible output).

Safety guarantees:
  - Never reads more than `_MAX_TRANSCRIPT_BYTES` from disk.
  - Rejects symlinks and non-regular files (device files, sockets).
  - Handles both string content (typed prompt) and list content
    (multi-modal prompts, tool results).

Performance:
  - Iterates lines from the END of the file in 64 KB chunks via a
    generator. Long-session transcripts (multi-MB) only pay for the
    bytes we actually consume, not the full file. The previous
    implementation called `splitlines()` on a 20 MB tail every hook
    invocation — ~150-300k string allocations per turn.
"""
from __future__ import annotations

import json
from collections.abc import Iterator
from pathlib import Path

_MAX_TRANSCRIPT_BYTES = 20 * 1024 * 1024  # 20 MB hard cap
_CHUNK_SIZE = 64 * 1024
# Hard cap per individual line. The transcript-wide cap above bounds
# total bytes read, but a single pathological line (corrupt write, JSON
# blob with embedded base64, or upstream bug) could otherwise grow
# unbounded inside the chunked reverse walk's carry buffer. 16 MB is
# orders of magnitude beyond any legitimate prompt or response.
_MAX_LINE_BYTES = 16 * 1024 * 1024


def _iter_tail_lines_reversed(transcript_path: str) -> Iterator[str]:
    """Yield non-empty lines from the end of `transcript_path`, in reverse.

    Reads in `_CHUNK_SIZE` chunks from the file's tail backwards. Stops
    when the caller breaks, when the start of the file is reached, or
    when `_MAX_TRANSCRIPT_BYTES` have been consumed (whichever comes
    first). Lines are decoded UTF-8 with replacement.

    Trailing partial line (file mid-write, missing final \\n) is dropped
    safely: the function locates the last `\\n` in the initial tail
    chunk and treats anything after it as incomplete.
    """
    try:
        path = Path(transcript_path).resolve()
    except (OSError, RuntimeError):
        return

    if not path.exists():
        return

    try:
        if path.is_symlink() or not path.is_file():
            return
        st = path.stat()
    except OSError:
        return

    size = st.st_size
    if size == 0:
        return

    try:
        with path.open("rb") as f:
            # ---- Step 1: locate end of last complete line. ----
            # Read up to one chunk at the tail to find the last `\n`.
            # Anything past it is a mid-write partial — we discard it.
            initial_size = min(_CHUNK_SIZE, size)
            f.seek(size - initial_size)
            tail = f.read(initial_size)
            last_nl = tail.rfind(b"\n")

            if last_nl == -1:
                # No newline in the tail. If the file fits in one chunk,
                # it is a single partial line — nothing complete to yield.
                # If larger, treat the whole tail as partial and start
                # the reverse walk before it.
                if size <= initial_size:
                    return
                effective_size = size - initial_size
            else:
                effective_size = (size - initial_size) + last_nl + 1

            # ---- Step 2: walk backwards in chunks, yielding lines. ----
            pos = effective_size
            buf = b""  # carryover: bytes that may continue a line into the next chunk
            bytes_read = 0

            while pos > 0 and bytes_read < _MAX_TRANSCRIPT_BYTES:
                read_size = min(_CHUNK_SIZE, pos, _MAX_TRANSCRIPT_BYTES - bytes_read)
                pos -= read_size
                f.seek(pos)
                data = f.read(read_size)
                bytes_read += read_size

                combined = data + buf
                parts = combined.split(b"\n")
                # parts[0] is a partial line at the front of `combined`
                # (it continues earlier in the file). Carry it — but
                # cap its size so a single pathological line cannot
                # accumulate unbounded memory across iterations.
                buf = parts[0]
                if len(buf) > _MAX_LINE_BYTES:
                    # Discard the partial line: it's larger than any
                    # legitimate JSONL entry. Any complete line older
                    # than this is unrecoverable from this position.
                    buf = b""
                # parts[1:] are complete lines, in original (forward) order.
                # Yield them in reverse so the caller sees newest first.
                # Skip lines that exceed _MAX_LINE_BYTES — they're either
                # corrupt or so large that JSON parsing downstream would
                # be a denial-of-service in itself.
                for line_bytes in reversed(parts[1:]):
                    if len(line_bytes) > _MAX_LINE_BYTES:
                        continue
                    line = line_bytes.decode("utf-8", errors="replace")
                    if line.strip():
                        yield line

            # ---- Step 3: file's first line, if we walked all the way. ----
            # If pos hit 0, the leftover `buf` is the start of the file.
            # If we stopped at the cap, `buf` is a partial line and is
            # discarded — same defensive behavior as the previous
            # implementation, which also dropped partial lines after seek.
            if pos == 0 and buf and len(buf) <= _MAX_LINE_BYTES:
                line = buf.decode("utf-8", errors="replace")
                if line.strip():
                    yield line
    except OSError:
        return


def extract_text_from_blocks(content) -> str:
    """Extract visible text from a content field.

    Accepts:
      - str: returned as-is.
      - list of blocks: concatenates all `type: "text"` blocks. Ignores
        thinking, tool_use, tool_result, image, and any other block type.

    Returns "" if nothing extractable.
    """
    if isinstance(content, str):
        return content
    if not isinstance(content, list):
        return ""
    texts = []
    for block in content:
        if isinstance(block, dict) and block.get("type") == "text":
            t = block.get("text", "")
            if isinstance(t, str):
                texts.append(t)
    return "\n".join(texts)


def extract_last_user_prompt(transcript_path: str) -> str:
    """Walk transcript backwards, return the most recent user prompt.

    Supports both string-content prompts (typed) and multi-modal prompts
    (list of text + image blocks). Skips user entries whose content is
    PURELY tool_result payloads (tool returns are not user prompts).

    Returns "" if not found or path invalid.
    """
    for line in _iter_tail_lines_reversed(transcript_path):
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue

        if event.get("type") != "user":
            continue

        message = event.get("message")
        if not isinstance(message, dict):
            continue

        content = message.get("content")
        text = extract_text_from_blocks(content)
        if text.strip():
            return text

    return ""


def extract_last_assistant_text(transcript_path: str) -> str:
    """Fallback: extract last assistant visible text from transcript.

    Used only when the Stop hook input does not provide
    last_assistant_message. Uses the same bounded read as
    extract_last_user_prompt.
    """
    for line in _iter_tail_lines_reversed(transcript_path):
        try:
            event = json.loads(line)
        except json.JSONDecodeError:
            continue

        if event.get("type") != "assistant":
            continue

        message = event.get("message")
        if not isinstance(message, dict):
            continue

        content = message.get("content")
        text = extract_text_from_blocks(content)
        if text.strip():
            return text

    return ""
