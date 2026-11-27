# Lesson plan: Release 1

Release 1 is Workflow 1 (supplier bill) and Workflow 4 (daily briefing).
Plan: 35 lessons and 2 spikes in 6 phases, about 13 weeks at 40 hours a week.
Week numbers are estimates, not deadlines. Requirement IDs refer to
docs/requirements.md, ADR IDs to docs/architecture.md.

## How a lesson runs

1. Concept: what the idea is and why this design was chosen.
2. Exercise: Rijo writes the code, with hints on request.
3. Tests: the "done when" checks pass locally and in CI.
4. Record: update docs/progress.md, add an ADR if a decision changed.

## Running alongside the code, from week 1

- Collect real supplier bills and emails for the evaluation set.
  Target: 100 bills and 300 emails by the start of Phase 3 (NFR-14).
- Interview 5 business owners or bookkeepers about how they handle bills.
- Set up the development Google Workspace on a subdomain or separate domain,
  and a QuickBooks Online sandbox company.

## Phase 0: Foundations (week 1)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 1 | Repository skeleton: Docker Compose with PostgreSQL, a FastAPI service and an Axum service, each with a health check | Monorepo layout, Cargo and uv basics | `docker compose up` starts all three, both health checks return 200 |
| 2 | CI with GitHub Actions: lint, format and test for Python and Rust | CI pipelines, caching | A failing test blocks a pull request |
| 3 | Terraform: dev project, Artifact Registry, both services on Cloud Run | Terraform state, modules, service accounts | `terraform apply` from empty gives two live health checks (NFR-17) |
| 4 | Database v1: `app` and `gateway` schemas, dbmate, row-level security on organizations | RLS, database roles, migrations | A test proves org A cannot read org B (NFR-01, NFR-02) |
| 5 | Tracing: one OpenTelemetry trace across Python and Rust | Trace context propagation | A request through both services shows as one trace (NFR-11) |

## Spikes (week 2, half a day each)

| # | Question | Done when |
|---|----------|-----------|
| S1 | Does a Gmail Internal app in the dev Workspace keep a long-lived refresh token for read and label scopes? | A token still works after 8 days, result written to docs/adr |
| S2 | Does the QuickBooks sandbox support vendor list, bill query and bill create as production does? | All three calls succeed from a script, limits noted |

## Phase 1: Gateway core in Rust (weeks 2 to 4)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 6 | Rust through Axum: routes, extractors, shared state, typed errors | Ownership, Result, traits, `thiserror` | A typed JSON endpoint with error mapping and unit tests |
| 7 | SQLx: queries, transactions, compile-time checked SQL | Async Rust, Tokio, connection pools | Integration tests run against a real PostgreSQL in CI |
| 8 | Caller identity: verify the orchestrator's Google-signed service token | JWT verification, middleware with Tower | Calls without a valid token get 401 |
| 9 | Audit log: append-only record of every call, success or failure | Middleware ordering, database permissions | Every request produces one audit row, updates are denied (FR-24, NFR-10) |
| 10 | Idempotency: key check, stored result, replay | Transactions, unique constraints | Same key twice returns the stored result and runs once (NFR-07) |
| 11 | Credential vault: envelope encryption with Cloud KMS | Encryption at rest, secrets in memory | Tokens are unreadable in the database, never appear in logs (NFR-03) |
| 12 | Tool pipeline: tool registry, the six checks in order, OpenAPI contract | Enums, traits, `utoipa`, contract tests | A fake tool passes all checks, Python client is generated from the contract (NFR-06) |

## Phase 2: Integrations and approvals (weeks 4 to 6)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 13 | OAuth connect flow with a per-organization client | OAuth 2.0 code flow, token refresh | The dev Workspace mailbox connects and disconnects (FR-03) |
| 14 | `MailboxConnector` trait and the Gmail connector: list new, get message | Trait objects, HTTP clients, fake servers for tests | New mail is listed once, tests run against a fake Gmail (FR-05, FR-07) |
| 15 | Attachments to Cloud Storage, and applying labels | Streaming, storage clients | A PDF lands under the organization's prefix, the email is labeled |
| 16 | QuickBooks: OAuth, vendor cache, bill query | Caching, rate limits | Vendors are cached and refreshed, read calls are counted (NFR-20) |
| 17 | Drive upload | Reuse of the connector pattern | A document is filed in the configured folder (FR-11) |
| 18 | Approvals: table, state machine, payload hash, Firebase token verification | State machines with enums, auth middleware | Illegal state changes are rejected, only an approver can decide (FR-15, ADR-06) |
| 19 | Approved writes: `qbo.create_bill` and `qbo.create_vendor` | Putting the pieces together | A bill posts once with a matching approval and never without one (FR-18) |

## Phase 3: Orchestrator and AI (weeks 6 to 9)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 20 | Orchestrator API: Firebase sign-in, organizations, memberships, RLS per request | Multi-tenant API design | Role checks and the isolation test pass (FR-01, FR-02) |
| 21 | Model interface with two provider adapters | Provider abstraction, structured output | The same call runs on two providers by configuration (NFR-16) |
| 22 | Evaluation harness over the labeled set | Evaluation design, metrics, regression tests | One command reports accuracy per field and per provider (NFR-14) |
| 23 | Email classification step | Prompt design, thresholds, the "unsure" path | Recall of 98% or more on the set (FR-06, NFR-15) |
| 24 | Bill extraction with confidence and source location | Multimodal input, Pydantic validation | 95% or more exact match per required field (FR-08 to FR-10) |
| 25 | Validation in code: arithmetic, vendor match, duplicates | Decimal arithmetic, trigram search | Bad totals and duplicates are caught in tests (FR-12 to FR-14) |
| 26 | LangGraph workflow with PostgreSQL checkpoints and a pause for approval | Durable graphs, interrupts, resume | Killing the worker mid-run loses nothing (NFR-09) |
| 27 | Scheduling: polling, Cloud Tasks retries, run caps | Queues, backoff, limits | A stuck run stops at its cap and becomes a failed task (FR-19, NFR-08) |
| 28 | Hostile input suite: emails and documents that try to redirect the workflow | Prompt injection, threat modeling | No hostile sample causes a tool call it should not (NFR-05) |

## Phase 4: Dashboard and briefing (weeks 9 to 11)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 29 | Next.js app with Firebase sign-in and organization switching | App Router, auth on the client and server | A signed-in user sees only their organization |
| 30 | Approvals screen: document beside fields, edit, approve or reject to the gateway | Forms, optimistic UI, calling two backends | An edit creates a new payload hash, approval posts the bill (FR-16, FR-17) |
| 31 | Overview, Activity and Settings | Data fetching, tables | Every run shows steps, status, duration and cost (FR-22, FR-23) |
| 32 | Daily briefing: generation, schedule, email from a Logic Shades address | Transactional email, domain authentication | The briefing arrives at the set time with correct counts (FR-20, FR-21) |

## Phase 5: Production readiness (weeks 11 to 13)

| # | Lesson | You learn | Done when |
|---|--------|-----------|-----------|
| 33 | Service levels, dashboards and alerts | SLIs, SLOs, error budgets | The four indicators are measured and an alert fires in a drill (NFR-12) |
| 34 | Cost tracking per run and per organization, operator pause | Usage accounting | Cost per bill is reported, pause stops all processing (FR-25, NFR-13) |
| 35 | Prod environment, deploy and rollback, runbook, data export and delete | Release engineering, incident response | Prod is built from Terraform, a rollback drill succeeds (NFR-18, NFR-19) |

## Release 1 exit check

- A bill emailed to the dev Workspace mailbox reaches QuickBooks sandbox after
  an approval in the dashboard, with one trace and a full audit trail.
- All FR and NFR items for Release 1 have a passing test or a recorded measurement.
- The evaluation report meets NFR-15 on the labeled set.
- A 5-minute demo runs on real bills from the evaluation set.

## After Release 1

- Release 2: overdue invoice reminders, Gmail send scope, forwarding connector.
- Release 3: customer inquiry replies, pgvector retrieval, MCP server on the gateway.
