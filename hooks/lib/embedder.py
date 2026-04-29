"""
embedder.py - Semantic scoring via embedding similarity.

Uses fastembed (ONNX-based, ~100 MB) to embed incoming text and compare
against anchor prototypes. This replaces keyword matching with semantic
similarity, making scoring language-agnostic and domain-adaptive.

If fastembed is not installed, every function in this module returns
None, signalling that the caller should fall back to structural scoring.

Design contract:
  - Never raises. All errors return None or 0.0.
  - Anchors are embedded once on first use, cached in memory for the
    process lifetime.
  - Embedding a single string is cheap (~20-50ms on CPU). Safe for hooks.
"""
from __future__ import annotations

import math
import threading
from typing import Sequence

try:
    from fastembed import TextEmbedding  # type: ignore
    _FASTEMBED_AVAILABLE = True
except Exception:
    TextEmbedding = None  # type: ignore
    _FASTEMBED_AVAILABLE = False


# Default multilingual model. Small, fast, supports 50+ languages.
# paraphrase-multilingual-MiniLM-L12-v2 is a well-known baseline.
# fastembed's default catalog uses slightly different names; we stick
# with one known to be packaged and small.
_DEFAULT_MODEL = "sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2"

_model_lock = threading.Lock()
_model: object | None = None
# Cache keyed by the tuple of anchor strings so the identity is content-
# based (not the Python id() of the list object, which is unstable
# across reimports and dynamically constructed lists).
_anchor_cache: dict[tuple, list[list[float]]] = {}


def is_available() -> bool:
    """Return True if the semantic scorer can run on this machine."""
    return _FASTEMBED_AVAILABLE


def _get_model():
    """Lazily instantiate the embedding model (thread-safe)."""
    global _model
    if _model is not None:
        return _model
    if not _FASTEMBED_AVAILABLE:
        return None
    with _model_lock:
        if _model is None:
            try:
                _model = TextEmbedding(model_name=_DEFAULT_MODEL)
            except Exception:
                # Fall back to whatever default model fastembed exposes
                try:
                    _model = TextEmbedding()
                except Exception:
                    return None
    return _model


def _embed(texts: Sequence[str]) -> list[list[float]] | None:
    """Return a list of embedding vectors, or None on failure."""
    model = _get_model()
    if model is None:
        return None
    try:
        # fastembed returns a generator of numpy arrays
        vectors = [list(v) for v in model.embed(list(texts))]  # type: ignore
        return vectors
    except Exception:
        return None


def _cosine(a: list[float], b: list[float]) -> float:
    """Cosine similarity in [-1, 1]. Returns 0 on dimension mismatch."""
    if len(a) != len(b):
        return 0.0
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(x * x for x in b))
    if na == 0.0 or nb == 0.0:
        return 0.0
    return dot / (na * nb)


def _get_anchor_embeddings(anchors: Sequence[str]) -> list[list[float]] | None:
    """Embed a sequence of anchors once, cached by content identity.

    Using the tuple of anchor strings as the key means:
      - the same module-level list always hits the cache;
      - two lists with identical content share the cache;
      - dynamically-constructed lists don't silently collide on id()
        reuse after gc.
    """
    key = tuple(anchors)
    if key in _anchor_cache:
        return _anchor_cache[key]
    vectors = _embed(list(anchors))
    if vectors is None:
        return None
    _anchor_cache[key] = vectors
    return vectors


def score_affinity(
    text: str,
    positive_anchors: Sequence[str],
    negative_anchors: Sequence[str],
) -> int | None:
    """Return an integer score in [0, 100] expressing affinity to positive
    vs negative anchors, or None if semantic scoring is unavailable.

    Semantics:
      - Embed `text` once.
      - Compute max cosine similarity against positive anchors -> sim_pos
      - Compute max cosine similarity against negative anchors -> sim_neg
      - Score is the normalized lead of positive over negative,
        rescaled to [0, 100].

    A text that is clearly aligned with positive anchors and clearly
    distant from negative anchors scores near 100. The opposite case
    scores near 0. Ambiguous inputs converge near 50.
    """
    if not text or not text.strip():
        return 0
    if not positive_anchors or not negative_anchors:
        return None

    text_vec = _embed([text])
    if text_vec is None:
        return None
    t = text_vec[0]

    pos_vecs = _get_anchor_embeddings(positive_anchors)
    neg_vecs = _get_anchor_embeddings(negative_anchors)
    if pos_vecs is None or neg_vecs is None:
        return None

    sim_pos = max(_cosine(t, v) for v in pos_vecs)
    sim_neg = max(_cosine(t, v) for v in neg_vecs)

    # Rescale lead into [0, 100]. Clamp extremes.
    # sim_pos - sim_neg naturally sits in [-1, 1]; remap to [0, 100].
    lead = sim_pos - sim_neg
    # Slight shaping: push ambiguous cases (small lead) toward mid-range,
    # while clear cases move assertively toward 0 or 100.
    shaped = (lead + 1.0) / 2.0  # [0, 1]
    score = int(round(shaped * 100))
    return max(0, min(100, score))
