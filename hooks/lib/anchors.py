"""
anchors.py - Prototype examples used by the semantic scorer.

These are NOT keywords. They are semantic prototypes:
the scorer embeds them once, then compares incoming prompts/responses
against the full set via cosine similarity. Adding a new technical
domain means adding 1-2 representative examples, not a keyword list.

All anchors are written in English. Multilingual embedding models
(e.g. paraphrase-multilingual or e5-multilingual) map semantically
equivalent inputs in other languages near the English anchor.

Keep the sets small (15-25 each). Too many anchors add noise.
"""

# ---------------------------------------------------------------------------
# PROMPT ANCHORS
# ---------------------------------------------------------------------------

SIMPLE_PROMPT_ANCHORS = [
    "What time is it?",
    "Hello.",
    "Thanks.",
    "OK, continue.",
    "How are you?",
    "Good morning.",
    "Yes, that works.",
    "Run the tests.",
    "Fix the typo on line 12.",
    "List files in this directory.",
    "Show me the README.",
    "What does this command do?",
    "Rename this variable.",
    "Open the config file.",
    "Commit these changes.",
]

COMPLEX_PROMPT_ANCHORS = [
    "Design a modular architecture for a real-time event processing pipeline. "
    "Analyze trade-offs between Kafka, Redis Streams, and NATS. Evaluate the "
    "main risks of each approach and recommend an implementation with edge "
    "cases considered.",

    "Refactor this authentication module to support multi-tenancy. Identify "
    "the impact on existing callers, risks of each migration strategy, and "
    "provide a phased rollout plan.",

    "I have a production backend running Node.js with PostgreSQL that needs "
    "to scale. Compare vertical and horizontal sharding strategies, evaluate "
    "the operational risks of each, and recommend the right path.",

    "Investigate why our p99 latency doubled after the last deploy. Walk "
    "through hypothesis tree, propose instrumentation, and structure the "
    "root-cause analysis.",

    "Plan the migration from a REST monolith to microservices. Consider data "
    "consistency, service boundaries, observability, deployment strategy, "
    "and failure modes under partial outages.",

    "Evaluate whether we should adopt GraphQL over REST for our public API. "
    "Consider schema evolution, caching, client complexity, and operational "
    "costs.",

    "Analyze the security posture of this authentication flow. Identify "
    "trust boundaries, potential attack vectors, and propose specific "
    "mitigations with acceptance criteria.",

    "Design the schema migration strategy for adding a new tenant_id column "
    "to a 500M-row table, preserving zero downtime and backward compatibility.",

    "Propose an observability strategy for this distributed system. Define "
    "SLIs, SLOs, tracing boundaries, and the alert threshold rationale.",

    "Benchmark the performance trade-offs between streaming SSE and "
    "WebSockets for real-time notifications. Account for reconnection "
    "semantics, proxy compatibility, and resource cost at 100k concurrent "
    "clients.",

    "Compare Rust async runtimes (tokio vs async-std vs smol) for a "
    "high-throughput network service. Consider ecosystem maturity, "
    "performance characteristics, and interoperability constraints.",

    "Architect a rate-limiting system that handles burst traffic, multiple "
    "tenant tiers, and graceful degradation. Describe the data structures, "
    "failure modes, and cross-region consistency.",

    "Walk through the full incident response for a data breach suspected in "
    "our Kafka cluster. Include detection, containment, communication, and "
    "post-mortem steps.",

    "Evaluate the feasibility of moving our training pipeline from PyTorch "
    "to JAX. Analyze the hardware, library, and engineering implications.",

    "Design a feature flag system that supports gradual rollouts, A/B tests, "
    "kill switches, and per-tenant overrides. Consider consistency, "
    "observability, and operational overhead.",
]


# ---------------------------------------------------------------------------
# RESPONSE ANCHORS
# ---------------------------------------------------------------------------

SHALLOW_RESPONSE_ANCHORS = [
    "Use Kafka. It's the best choice.",
    "Depends on your case.",
    "Try horizontal sharding first.",
    "Just use Postgres for now.",
    "It should work. Let me know if it breaks.",
    "Follow best practices.",
    "Run the migration and see what happens.",
    "Add an index and benchmark.",
    "Yeah, that approach is fine.",
    "Split it into microservices.",
    "Rewrite it in Rust.",
    "Add caching, it'll be fast enough.",
    "Use a managed service.",
    "Throw more CPU at it.",
    "Use Option A. Simpler.",
]

DEEP_RESPONSE_ANCHORS = [
    # Each deep anchor demonstrates multi-section structure, trade-off
    # analysis, explicit risks, and concrete actionable steps.
    """## Analysis of alternatives

### Option A: Kafka
- **Pros**: high throughput, durable persistence, mature ecosystem
- **Cons**: operationally complex, requires ZooKeeper or KRaft
- **Risk**: operational complexity may delay time-to-production
- **Mitigation**: use managed service (MSK, Confluent Cloud)

### Option B: Redis Streams
- **Pros**: low latency, operationally simple
- **Cons**: limited durability, memory-bound
- **Risk**: event loss if Redis fails without replication
- **Mitigation**: replication plus AOF fsync

## Recommendation

Kafka, because expected volume exceeds 100k events/sec. If actual volume
drops below 10k events/sec, Redis Streams would be more cost-efficient.

## Failure modes identified

1. Back-pressure: slow consumers cause lag. Mitigation: monitoring + autoscale
2. Poison messages: malformed events block partition. Mitigation: DLQ pattern
3. Schema evolution: breaking changes break consumers. Mitigation: Schema Registry

## Impact on codebase

Introduce EventBus abstraction to decouple producers/consumers from the
concrete broker. Gradual migration with feature flag and shadow writes
over a 2-week window.""",

    """## Approach

Start with instrumentation before refactoring. Premature optimization without
measurement is guaranteed to miss the real bottleneck.

### Phase 1 (week 1): Measure
- Enable `pg_stat_statements` and review top queries by total_time
- Run `pgbadger` against logs to identify N+1 patterns
- Add APM traces at controller boundaries (Datadog, New Relic)
- Capture baseline: p50/p95/p99 per endpoint

### Phase 2 (weeks 2-3): Low-risk wins
- Deploy PgBouncer in transaction-pooling mode
- Align Node pool size with PgBouncer pool_size
- Add missing indexes identified in Phase 1
- Rewrite obvious N+1 queries

Expected improvement: 3-10x without architecture changes.

### Phase 3 (months 1-2): Vertical + replicas
Promote the primary one tier. Add 2 read replicas. Route lag-tolerant
reads to replicas via an explicit routing predicate.

### Risks
- Session-sticky reads on read-after-write: use primary for writes +
  immediately following reads within the same request.
- Replica lag under write spikes: set max_replication_lag threshold
  and fall back to primary when exceeded.

### Rollback strategy
Every phase is independently revertable. Phase 1 is instrumentation only.
Phase 2's PgBouncer adds a hop that can be disabled in 30 seconds.""",

    """## Trust boundaries

The boundary sits at three points:
1. Client -> API gateway (TLS termination, auth token validation)
2. API gateway -> service (internal mTLS, service identity assertion)
3. Service -> database (credential rotation, least-privilege scope)

## Attack surface per boundary

### Client -> Gateway
- Token replay: mitigated by short-lived access tokens (15 min) plus
  rotating refresh tokens with device binding
- JWT tampering: signed with rotating keypair, verification against JWKS
- Parameter injection: schema validation before routing

### Gateway -> Service
- Identity spoofing: mTLS with service account certs rotated every 24h
- Replay of internal requests: request signatures with nonce + timestamp

### Service -> Database
- Credential leakage: secrets from vault, not environment variables
- Privilege escalation: per-service DB role with row-level scope

## Missing controls to add
- WAF rules for known bot signatures
- Rate limiter keyed by user_id, not just IP
- Audit log export to immutable storage

## Acceptance criteria

The flow is considered hardened when penetration testing confirms:
no unauthenticated endpoints reachable, token replay window < 60 seconds,
no secrets in any runtime log, and role transitions require explicit
re-authentication.""",

    """## Context and constraints

Assuming: OLTP workload, read/write ratio ~70/30, single-region deployment,
downtime budget of <5 minutes, rollback window of 24 hours.

## Strategy: online schema change

### Step 1: Add column as nullable
```sql
ALTER TABLE users ADD COLUMN tenant_id BIGINT NULL;
```
Non-blocking on Postgres 11+. Zero locks held beyond metadata.

### Step 2: Backfill in batches
Background worker, 1000 rows per batch, 100ms pause between batches.
Monitor replication lag; pause if it exceeds 5 seconds.

### Step 3: Enforce constraint
```sql
ALTER TABLE users ADD CONSTRAINT users_tenant_id_not_null
  CHECK (tenant_id IS NOT NULL) NOT VALID;
ALTER TABLE users VALIDATE CONSTRAINT users_tenant_id_not_null;
```
`NOT VALID` + `VALIDATE` separates the add from the check. The validation
scan is still long but non-blocking.

### Step 4: Flip to NOT NULL
Only after validation succeeds:
```sql
ALTER TABLE users ALTER COLUMN tenant_id SET NOT NULL;
```

## Risks
1. Backfill worker crash mid-run: idempotent marker column allows resume
2. Application writes without tenant_id during migration: middleware
   enforces default value at insert time
3. Lock contention during VALIDATE: run during low-traffic window,
   monitor pg_stat_activity""",

    """## Recommendation

Do NOT migrate to microservices yet. The symptoms you describe (slow
deploys, fear of breaking unrelated features) are solvable with module
boundaries inside the monolith before paying microservices' tax.

### Why not microservices first

Distributed systems add: network latency between every boundary, eventual
consistency where transactions used to exist, operational surface 5-10x
larger, debugging across services, observability across hops. The cost
is justified only when the boundaries you need are organizational, not
just code-organizational.

### What to do first

1. **Modular monolith**: enforce domain boundaries with import rules and
   integration tests that fail if cross-domain imports appear
2. **Database schema separation**: each domain owns its tables, no
   cross-domain JOINs in application code
3. **Event bus internally**: domains communicate via in-process events
   with the same contracts they would over HTTP

### When to actually split

Only after:
- A domain has a team that owns its full lifecycle
- The domain's deployment cadence clashes with the rest of the codebase
- The domain has SLOs meaningfully different from neighbors
- Instrumentation is already structured around domain boundaries

### If split is decided

Split one domain at a time. Start with the least-critical, most-isolated
one (usually notifications or reporting). Measure operational cost before
splitting the next.""",
]
