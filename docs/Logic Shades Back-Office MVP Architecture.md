# Logic Shades Back-Office MVP: Architecture

Nov 26, 2026 · @rijo george · Draft 1

## At a glance

The orchestrator decides what should happen, and the gateway is the only component that holds credentials and reaches Gmail, Drive and QuickBooks.

&#91;embedded content: system architecture · 3 services, 3 dependencies\]

A person's approval goes from the dashboard straight to the gateway, so the orchestrator can request a write but can never authorize one.

## Components

The system is three deployable services and one database. The rule that shapes everything: the Python side decides what should happen, and only the Rust gateway can make it happen in a customer's accounts.

| Component | Language | Owns | Must never |
| --- | --- | --- | --- |
| Dashboard | TypeScript, Next.js | Screens for overview, approvals, activity and settings | Hold any credential other than the user's sign-in session |
| Orchestrator API | Python, FastAPI, LangGraph, Pydantic | Workflow runs, model calls, extraction, validation logic, briefings, the evaluation harness | Hold an OAuth token or password, or call Gmail, Drive or QuickBooks directly |
| Tool gateway | Rust, Axum, Tokio, SQLx | Integration connectors, credential vault, tool authorization, approvals, idempotency, audit log | Call a model, or act on a write without a matching approval |
| Database | PostgreSQL | All state, split into an `app` schema and a `gateway` schema | Let the orchestrator's database role read the credential tables |
| Object storage | Google Cloud Storage | Source documents, one prefix per organization | Be publicly readable |

The orchestrator runs as two processes from one codebase: an API that serves the dashboard, and a worker that executes workflow steps.

## Workflow 1 run

A supplier bill moves through eight steps, and the run is saved after each one so a restart resumes where it stopped.

1. **Poll.** A scheduler asks the gateway for new mail every 2 minutes per organization.
2. **Classify.** The orchestrator's model call labels the message: bill, not a bill, or unsure.
3. **Fetch.** The gateway stores the attachment in object storage and returns a reference.
4. **Extract.** A model call returns bill fields as a typed Pydantic object with confidence and source location per field.
5. **Validate.** Code checks arithmetic, matches the vendor and looks for duplicates using cached QuickBooks data.
6. **Propose.** The orchestrator asks the gateway to create an approval request holding the exact bill payload.
7. **Wait.** The run pauses. Nothing is running or billing while it waits.
8. **Post.** After approval, the gateway creates the bill in QuickBooks and the run records the bill ID.

Labeling the email and filing the document in Drive are low-risk actions and run without approval after step 5.

**Approval states**

| State | Meaning | Next |
| --- | --- | --- |
| Pending | Waiting for an owner or approver | Approved, Rejected, Expired |
| Approved | A user approved this exact payload | Executed, Failed |
| Rejected | A user rejected it with a reason | End |
| Expired | No decision within 14 days | End, shown in the briefing |
| Executed | The write succeeded | End |
| Failed | The write failed after retries | Operator review |

An approval is bound to a hash of the payload. If an approver edits a field, the edit creates a new payload and the approval covers that one. The gateway refuses any write whose payload hash does not match an approved request.

**Failure handling**

- A failed step is retried with backoff up to 5 times.
- External writes carry an idempotency key, so a retry cannot post a second bill.
- A run that exhausts retries becomes a failed task, visible in Activity and in the briefing.

## Tool gateway interface

The gateway exposes ten tools in Release 1 over an internal HTTP API with typed JSON requests and responses, described by one OpenAPI contract that both sides are tested against.

| Tool | Kind | Needs approval |
| --- | --- | --- |
| `mail.list_new` | Read | No |
| `mail.get_message` | Read | No |
| `mail.store_attachment` | Read, writes to our storage | No |
| `mail.apply_label` | Low-risk write | No |
| `drive.upload_file` | Low-risk write | No |
| `qbo.list_vendors` | Read, cached | No |
| `qbo.find_bills` | Read | No |
| `qbo.create_vendor` | Write | Yes |
| `qbo.create_bill` | Write | Yes |
| `notify.send_briefing` | Send from a Logic Shades address | No |

**Checks on every call, in order**

1. The caller is the orchestrator's service identity.
2. The organization in the request exists, is not paused, and has the integration connected.
3. The tool is enabled for that organization and workflow.
4. For a write that needs approval: an approved request exists and its payload hash matches.
5. The idempotency key has not been used. If it has, the stored result is returned.
6. The call runs, and the audit record is written whether it succeeds or fails.

**Other gateway endpoints**

- Connect and disconnect an integration (OAuth redirect handling).
- Create an approval request (called by the orchestrator).
- Approve or reject (called by the dashboard with the user's sign-in token, which the gateway verifies itself).

The orchestrator cannot approve anything. It can only ask, and a person answers through the gateway.

**MCP.** In Release 1 the workflow calls tools from fixed code, so plain HTTP is enough. An MCP server exposing the same tools is added when a model starts choosing tools itself, planned for Release 3.

## Mailbox connectors

The `mail.*` tools are one interface with a connector behind it per connection method, so the workflow does not know which method a customer uses.

| Priority | Connector | Credential the gateway stores | New mail arrives by | Release |
| --- | --- | --- | --- | --- |
| 1 | Gmail Internal app | OAuth client per organization, plus a refresh token per connected mailbox | Polling Gmail's history every 2 minutes | 1 |
| 2 | Forwarding | None | An inbound email service posting to the gateway | When first needed |
| 3 | IMAP and SMTP | Username and app password | Polling | On customer demand |

- In Rust the interface is a trait, `MailboxConnector`, with one implementation per row. This is the first trait-based design lesson.
- A connector reports its capabilities. Forwarding cannot label or send, so `mail.apply_label` is skipped for those organizations.
- Polling is chosen over Gmail push for Release 1. Push needs a notification topic in each customer's project, and 2-minute polling already meets the 5-minute intake requirement.
- Scopes requested for Gmail in Release 1: read and label only. Send is added in Release 2.

## Data model

Release 1 uses fourteen tables in two schemas, and each schema has one owning service. This is a conceptual model, and columns are defined in the lessons.

| Schema | Table | Purpose |
| --- | --- | --- |
| app | `organizations` | Customer businesses, with paused flag and settings |
| app | `users` | People, keyed by Firebase user ID |
| app | `memberships` | User, organization and role |
| app | `workflow_runs` | One row per run: workflow, status, source message, cost |
| app | `run_steps` | Step name, status, duration, tokens and trace ID |
| app | `documents` | Stored files and their storage reference |
| app | `bills` | Extracted fields, confidence, validation results, QuickBooks bill ID |
| app | `corrections` | Approver edits, feeding the evaluation set |
| gateway | `integrations` | Which systems an organization has connected, and status |
| gateway | `credentials` | Encrypted OAuth clients, tokens and passwords |
| gateway | `approvals` | Request, payload hash, state, who decided and when |
| gateway | `idempotency_keys` | Key, request hash and stored result |
| gateway | `audit_log` | Append-only record of every tool call and decision |
| gateway | `vendor_cache` | Cached QuickBooks vendors and accounts per organization |

- Every table except `users` carries `organization_id`, and row-level security policies enforce it.
- The orchestrator's database role has no access to `gateway.credentials` and read-only access to `gateway.approvals` and `gateway.audit_log`.
- LangGraph stores its checkpoints in its own tables in the `app` schema. `workflow_runs` and `run_steps` are the reporting view the dashboard reads.
- Migrations are plain SQL managed by dbmate, with one migration directory per schema.

## Security model

The design assumes the model can be tricked by a hostile email and makes sure that being tricked cannot move money or leak another customer's data.

| Threat | Control |
| --- | --- |
| An email tells the model to pay a different account or approve itself | Email text is only ever data in a prompt. The model returns typed fields, code decides the next step, and every QuickBooks write needs a human approval of the exact payload |
| The orchestrator is compromised | It holds no credentials, cannot approve, and can call only the gateway's tool list. The gateway re-checks organization, tool and approval on every call |
| One customer sees another's data | `organization_id` on every row, row-level security in PostgreSQL, a storage prefix per organization, and an isolation test in CI |
| Stolen database backup | Credentials are encrypted with a key in Cloud KMS that only the gateway's service account can use |
| A retry posts a bill twice | Idempotency keys checked in the gateway before any external write |
| A runaway run burns money | Per-run caps on steps, tokens and cost, enforced by the orchestrator |
| An operator misuses access | Operators cannot approve for a customer, and every operator action is in the audit log |
| A forged approval | The gateway verifies the user's Firebase token and role itself, and does not trust the orchestrator's word |

**Service identity.** The orchestrator calls the gateway with a Google-signed service identity token. The gateway is not reachable from the public internet except for its OAuth redirect and approval endpoints.

## Deployment and operations

Everything runs on Google Cloud in one region, defined in Terraform, with a dev and a prod environment in separate projects.

| Need | Service |
| --- | --- |
| Dashboard, orchestrator API, worker, gateway | Cloud Run, one service each |
| Database | Cloud SQL for PostgreSQL |
| Documents | Cloud Storage |
| Step execution and retries | Cloud Tasks |
| Mail polling and the daily briefing | Cloud Scheduler |
| Credential encryption key | Cloud KMS |
| Application secrets | Secret Manager |
| Sign-in | Firebase Authentication |
| Container images | Artifact Registry |
| Traces, metrics and logs | OpenTelemetry exporting to Cloud Trace and Cloud Monitoring |
| Briefing email | A transactional email provider, to be chosen |

- **Local development:** Docker Compose runs PostgreSQL, the orchestrator and the gateway, with fake Gmail and QuickBooks servers for tests.
- **CI:** GitHub Actions runs Python tests, Rust tests, the contract test between them, the tenant isolation test and the evaluation set on every change.
- **Deploy:** one command per environment. Rollback is a redeploy of the previous image.
- **Tracing:** one trace follows a bill from poll to post across Python and Rust, with the trace ID stored on the run.

**Service levels to measure from the first end-to-end run**

| Indicator | Starting target |
| --- | --- |
| Time from email arrival to approval request | 95% within 10 minutes |
| Runs that finish without operator action | 90% |
| Time from approval to bill posted | 99% within 2 minutes |
| Gateway availability | 99.5% monthly |

These targets are proposals to tune once real numbers exist.

## Decision records

Twelve architecture decisions are made or proposed so far. Each becomes a short record in the repository's `docs/adr` folder when the project starts.

| ID | Decision | Status | Main reason |
| --- | --- | --- | --- |
| ADR-01 | First product is the email-to-invoice workflow plus a daily briefing | Accepted | One demonstrable problem that exercises every integration |
| ADR-02 | Python for orchestration, Rust for the tool gateway, from Release 1 | Accepted | Real security boundary, and Rust is a learning goal with no deadline pressure |
| ADR-03 | Workflows are fixed graphs with model calls inside steps, not free-roaming agents | Proposed | Testable, cheaper, and harder to hijack through email content |
| ADR-04 | Model provider is open, behind one interface, chosen per task by evaluation | Accepted | Avoids lock-in and makes accuracy claims measurable |
| ADR-05 | Mailbox connection order: Gmail Internal app, then forwarding, then IMAP | Accepted | No Google review for pilots, with a fallback for every provider |
| ADR-06 | Approvals are owned and verified by the gateway, bound to a payload hash | Accepted | The orchestrator cannot forge or alter an approved action |
| ADR-07 | One PostgreSQL database, two schemas, one owning service each | Proposed | Simple to run solo, with a clear split for a later team |
| ADR-08 | Orchestrator calls the gateway over HTTP with an OpenAPI contract. MCP added in Release 3 | Proposed | Fixed workflows do not need a tool protocol yet |
| ADR-09 | Poll Gmail every 2 minutes instead of using push notifications | Proposed | Avoids per-customer notification setup and meets the intake target |
| ADR-10 | Google Cloud with Cloud Run, defined entirely in Terraform | Accepted | Matches existing hosting and the platform learning goal |
| ADR-11 | Plain SQL migrations with dbmate, no ORM | Proposed | Works for both Python and Rust, and keeps row-level security explicit |
| ADR-12 | Low-risk actions run without approval. Every QuickBooks write needs one | Accepted | Removes friction where a mistake is harmless and reversible |

## Open questions

Two decisions are settled, and four items remain. None of them blocks the lesson plan.

- **Settled:** a Google Workspace organization will be created for development, on a subdomain or separate domain so Logic Shades mail stays on Spacemail.
- **Settled:** approvals live in the Rust gateway (ADR-06).

* [ ] The five decisions still marked Proposed (ADR-03, 07, 08, 09, 11) are treated as accepted unless you object.
* [ ] Which transactional email provider sends the briefing? Decided in the briefing lesson.
* [ ] Spike: confirm in practice that a Gmail Internal app's refresh token outlives 7 days. Google's documentation says it does. The token is still revoked when the user changes their password, when the organization enforces session-length policies, or after six months unused, so the gateway must detect a revoked token, mark the mailbox disconnected and alert the owner.
* [ ] Spike: run vendor list, bill create and a bill query filtered by vendor and bill number against the QuickBooks sandbox. Intuit documents the same API for sandbox and production with a different base URL. Still to check outside the sandbox: which QuickBooks Online plans include bills, and Intuit's steps for production keys.

The two verification items become the first technical spikes, done before any feature work depends on them.
