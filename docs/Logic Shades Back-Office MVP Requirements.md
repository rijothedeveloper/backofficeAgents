# Logic Shades Back-Office MVP: Requirements

Nov 26, 2026 · @rijo george · Draft 1

## Summary and decisions

The MVP is an email-to-invoice assistant: a supplier bill arrives in Gmail, is extracted and filed in Drive, validated against QuickBooks, approved by a person, and posted as a bill. A daily briefing reports what happened. It is built as two services: Python for orchestration and model work, and a Rust tool gateway for integrations, authorization and audit.

| Decision | Status | Detail |
| --- | --- | --- |
| First product | Settled | Email-to-invoice assistant (supplier bills) plus daily briefing |
| Delivery model | Settled | Managed service first; self-serve SaaS later |
| Model provider | Settled: open | All model calls go through one internal interface; provider chosen per task by evaluation results |
| Rust | Settled: from the start | The tool gateway is written in Rust from Release 1. Orchestration and model work stay in Python |
| Design partner | None yet | Build and evaluate against Logic Shades' own Google Workspace and a QuickBooks sandbox company |
| Demo date | None | Progress is gated by milestones, not dates |
| Weekly hours | Settled | 40 hours a week. Release 1 is estimated at 12 to 14 weeks, including learning Rust |

No code is written until this document and the architecture document are both agreed.

## Goals and non-goals

The project has two goals that carry equal weight, and each requirement below serves at least one of them.

**Commercial goal.** A working product that can be demonstrated on real mailboxes and a real QuickBooks company, good enough to win a first design partner and then three paying customers.

**Learning goal.** Hands-on practice in AI engineering, Rust, cloud engineering, platform engineering and SRE. Each technology enters when the product has a real need for it, so every lesson ends with something running.

**Non-goals for the MVP**

- Customer inquiry replies with retrieval (later release)
- Overdue invoice reminders (second release)
- AI chat screen and agent management screen
- Payments, payroll, accounting adjustments or any money movement
- Microsoft 365, WhatsApp, SMS and voice
- Self-serve signup, subscription billing and a mobile app
- Separate microservices per agent

## Users and roles

Three roles use the system, and the person who buys it is often not the person who approves bills.

| Role | Who | Can do |
| --- | --- | --- |
| Owner | Business owner or manager | Connect integrations, set approval rules, approve or reject, read briefings, invite members |
| Approver | Bookkeeper, office manager or outside accountant | Review extracted bills, correct fields, approve or reject |
| Operator | Logic Shades (Rijo) | Onboard a tenant, configure workflows, watch errors and costs. Cannot approve on a customer's behalf |

Every user belongs to one or more organizations. All data and every action are scoped to one organization.

## MVP scope

Release 1 contains two workflows. The four "agents" are stages of these workflows, coordinated by one orchestration graph.

**Workflow 1: supplier bill**

1. A new message arrives in the connected Gmail inbox.
2. The email stage classifies it. Messages that are not supplier bills are labeled and left alone.
3. The document stage extracts bill fields from the attachment or the email body.
4. The original file is saved to the organization's Drive folder.
5. The finance stage matches the vendor, checks for duplicates and validates totals against QuickBooks.
6. A draft bill appears in Approvals with the source document beside the extracted fields.
7. The approver corrects fields if needed, then approves or rejects.
8. On approval, the bill is created in QuickBooks and its due date is tracked. On rejection, nothing is written and the reason is recorded.

**Workflow 4: daily briefing**

Once a day, the owner receives a summary: bills processed, items waiting for approval, bills due in the next seven days, and anything that failed.

**Later releases**

| Release | Adds |
| --- | --- |
| 2 | Overdue customer invoice reminders (QuickBooks read, Gmail send) |
| 3 | Customer inquiry draft replies using business documents |
| 4 | Per-customer workflow configuration without code changes |

## Mailbox connection methods

A customer's mailbox is connected by the first method in this order that their email provider supports.

| Priority | Method | Works for | Google review | Built in |
| --- | --- | --- | --- | --- |
| 1 | Gmail through an Internal OAuth app in the customer's own Google Workspace organization | Google Workspace customers | None | Release 1 |
| 2 | Forwarding to a per-customer Logic Shades address | Any provider, including free Gmail and Spacemail | None | When the first customer needs it |
| 3 | IMAP and SMTP with a password | Providers with no OAuth option | None | Only when a paying customer needs it |

- Method 1 needs an OAuth client per customer, so the gateway stores client credentials per organization.
- Method 2 is one-way: no labeling, thread history or sending from the customer's address.
- Method 3 passwords are typed by the customer into the dashboard, encrypted at once, and never seen by the operator. An app password and a dedicated bills mailbox are preferred.
- Public Google verification is deferred until a move to self-serve SaaS.
- Building method 1 needs a Google Workspace organization for development and testing.

## Functional requirements

Each requirement is testable and has an ID that tests, lessons and architecture decisions will refer to.

| ID | Area | Requirement |
| --- | --- | --- |
| FR-01 | Tenancy | An operator can create an organization and invite an owner |
| FR-02 | Tenancy | An owner can invite members and assign the owner or approver role |
| FR-03 | Integrations | An owner can connect and disconnect Gmail, Google Drive and QuickBooks Online through OAuth |
| FR-04 | Integrations | Settings shows each integration's status and the time of its last successful call |
| FR-05 | Intake | The system detects new Gmail messages within 5 minutes of arrival |
| FR-06 | Intake | Each message is classified as supplier bill, not a bill, or unsure. Unsure goes to a person |
| FR-07 | Intake | A message is processed once, even if it is delivered or notified more than once |
| FR-08 | Extraction | The system extracts vendor name, bill number, bill date, due date, currency, subtotal, tax, total and line items from PDF and image attachments |
| FR-09 | Extraction | Each extracted field carries a confidence value and a pointer to where it was found in the document |
| FR-10 | Extraction | Missing or low-confidence required fields are flagged in the approval view |
| FR-11 | Filing | The source document is saved to a configured Drive folder with a consistent name |
| FR-12 | Validation | The vendor is matched to a QuickBooks vendor. No match produces a "new vendor" proposal for the approver |
| FR-13 | Validation | A bill is flagged as a possible duplicate when vendor and bill number match an earlier bill, or vendor, total and date match |
| FR-14 | Validation | Line totals, tax and grand total are checked arithmetically in code, not by the model |
| FR-15 | Approval | Every QuickBooks write requires an approval from a user with the owner or approver role |
| FR-16 | Approval | The approver sees the source document beside the extracted fields and can edit any field before approving |
| FR-17 | Approval | A rejection requires a reason and causes no external write |
| FR-18 | Posting | An approved bill is created in QuickBooks exactly once, and the QuickBooks bill ID is stored |
| FR-19 | Posting | A failed post is retried safely, and after the retry limit it appears as a failed task with the error |
| FR-20 | Tracking | Each bill's due date is tracked and appears in the briefing 7 days before it is due |
| FR-21 | Briefing | A daily briefing is generated at a time the owner sets and delivered in the dashboard and by email |
| FR-22 | Dashboard | Overview shows counts for the period: messages processed, bills extracted, approvals pending, failures |
| FR-23 | Dashboard | Activity shows every workflow run with its steps, status, duration and cost |
| FR-24 | Audit | Every external read and write, approval and configuration change is recorded with actor, time, organization and outcome |
| FR-25 | Operator | An operator can pause all processing for an organization with one action |
| FR-26 | Corrections | Fields edited by an approver are stored as corrections and added to the evaluation set |

## Non-functional requirements

Security and tenancy requirements apply from the first commit, because the system holds mail, financial documents and accounting access.

| ID | Area | Requirement |
| --- | --- | --- |
| NFR-01 | Tenancy | Every tenant-owned row carries `organization_id`, enforced by PostgreSQL row-level security as well as application code |
| NFR-02 | Tenancy | An automated test proves a user of one organization cannot read or act on another's data |
| NFR-03 | Secrets | OAuth tokens are encrypted at rest with a key held in a key management service, never logged, and never placed in a model prompt |
| NFR-04 | Least privilege | Each integration requests the narrowest scopes the enabled workflows need |
| NFR-05 | Untrusted input | Email and document content is treated as data. It cannot change which tools run, their targets or approval rules |
| NFR-06 | Tool authorization | Every tool call is checked against organization, workflow and approval state by code outside the model |
| NFR-07 | Idempotency | Every external write carries an idempotency key, so a retry cannot create a second bill or send a second email |
| NFR-08 | Limits | Each run has a cap on steps, model tokens and cost. Exceeding a cap stops the run and raises a failed task |
| NFR-09 | Durability | A workflow survives a process restart and resumes from its last completed step, including while waiting for approval |
| NFR-10 | Audit | Audit records are append-only and retained for at least 7 years (to confirm with an accountant) |
| NFR-11 | Observability | Every run produces one trace across API, orchestration, model calls and integration calls, using OpenTelemetry |
| NFR-12 | Service levels | Targets are defined and measured for intake delay, run success rate and approval-to-post time before the first pilot |
| NFR-13 | Cost | Model and cloud cost is recorded per run and per organization. Target is under $0.25 per processed bill |
| NFR-14 | Evaluation | A labeled set of at least 100 real bills and 300 emails exists before any accuracy figure is quoted |
| NFR-15 | Accuracy | On that set: bill classification recall of 98% or more, and 95% or more exact match per required field |
| NFR-16 | Model portability | Changing model provider for a task needs a configuration change and an evaluation run, not a code rewrite |
| NFR-17 | Infrastructure | All cloud resources are defined in Terraform. Nothing is created by hand in the console |
| NFR-18 | Delivery | Every change passes automated tests in CI before deployment. Deployment and rollback are each one command |
| NFR-19 | Data handling | A customer's data can be exported and deleted on request, and customer data is not used to train models |
| NFR-20 | QuickBooks usage | Vendor and account lists are cached, so read calls stay well inside Intuit's monthly allowance |

**External gates**

- Reading Gmail needs restricted scopes, which bring Google verification and a yearly security assessment for public apps that handle the data on a server. Internal apps inside a customer's own Workspace are exempt, which is why that is the first connection method. See [restricted scope verification](https://developers.google.com/identity/protocols/oauth2/production-readiness/restricted-scope-verification).
- QuickBooks Online meters read-type calls and caps the free tier at 500,000 a month. Writes are not metered. See the [Intuit App Partner Program FAQ](https://developer.intuit.com/app/developer/qbo/docs/get-started/partner-faq).

The cost, accuracy and retention figures are proposed starting targets, not measured values.

## Technology and learning plan

Each technology enters when a requirement calls for it, so the learning follows the product.

| Technology | Enters | Why then | Teaches |
| --- | --- | --- | --- |
| Python, FastAPI, Pydantic, PostgreSQL | Start | Fastest path to a working workflow | API design, schema design, row-level security |
| Docker, Terraform, Cloud Run, CI | Start | NFR-17 and NFR-18 from day one | Cloud and platform engineering |
| Firebase Authentication | Start | Roles and tenancy are needed before any data exists | Identity, token verification |
| LangGraph | First workflow | Durable steps and waiting for approval (NFR-09) | Orchestration, checkpointing |
| Model interface and evaluation harness | First extraction | NFR-14 to NFR-16 | AI engineering, evaluation |
| OpenTelemetry | First end-to-end run | NFR-11 and NFR-12 | Tracing, service levels, SRE practice |
| pgvector | Release 3 | Not needed until document retrieval | Retrieval |
| Next.js and TypeScript | Once the API for approvals is stable | The dashboard is the product surface | Frontend |
| Rust, Axum, Tokio, SQLx | Start, for the tool gateway only | The gateway is needed in every release, so it is built once in its final language | Systems programming, async, security boundaries |
| MCP | With the Rust gateway | The gateway exposes tools over one protocol | Tool protocol design |

**Where Rust gives an advantage.** The tool gateway is the component that holds OAuth tokens, checks authorization, enforces idempotency and writes the audit log. Moving it into its own process means the model-facing Python code never holds a credential. That is a real security boundary, and it is also small, well specified and already covered by tests by the time it is ported.

**Why now and not later.** The gateway is permanent, there is no demo date, and learning Rust is a goal in its own right. Building it in Python first would mean writing and testing it twice. The cost is a slower first end-to-end run, estimated at 4 to 6 extra weeks while Rust is new.

**Limits that keep Rust from slowing the product.** Rust is used for the gateway only. The gateway has a small typed interface: about six tools, a policy check, an idempotency check and an audit write. No workflow code calls Gmail, Drive or QuickBooks directly. If the first gateway tool is not working end to end within 4 weeks of starting it, we review the decision.

**Team growth.** One or two developers may join later. The two-service split gives a clear ownership line, and every interface between services is documented and contract-tested so a new developer can work on one side without learning the other.

## Assumptions and risks

The largest risk is building without a customer, so the plan substitutes real data and early conversations for a design partner.

| Risk | Effect | Response |
| --- | --- | --- |
| No design partner | Requirements reflect guesses about how small businesses handle bills | Use Logic Shades' own mailbox and bills as the first tenant. Interview 5 owners or bookkeepers before the architecture is frozen |
| No real bills to test on | Accuracy targets cannot be measured | Collect own supplier bills and receipts, plus bills donated by interviewees, into the evaluation set |
| Gmail verification | A customer cannot connect a mailbox to an unverified public app without warnings and limits | Avoided for pilots: connect through an Internal app in the customer's own Workspace, with forwarding as the fallback. See mailbox connection methods |
| QuickBooks already captures bills | The invoice workflow alone may not justify $750 a month | Position the price on the managed service across Workflows 1, 3 and 4. Test the price in interviews |
| Scope growth from learning goals | The product never reaches a demo | A technology is added only when its trigger in the plan is met |
| Prompt injection through email | An email could try to redirect a payment or approve itself | NFR-05 and NFR-06, plus a test suite of hostile emails and documents |
| Solo operator | One person builds, runs and supports every tenant | FR-25 pause control, alerts on failures, and a written runbook before the first pilot |

**Assumptions**

- Pilot customers use Google Workspace and QuickBooks Online, not QuickBooks Desktop.
- Bills arrive mainly as PDF or image attachments in English and US dollars.
- A person approves every bill in Release 1. Auto-approval rules come later, if at all.

## Open questions and next steps

The questions that blocked the architecture document are answered. One item is assumed and can still be changed.

- **QuickBooks for development:** the Intuit sandbox first, then a free or trial QuickBooks Online company when real-account testing is needed.
- **Briefing email:** sent from a Logic Shades address. Release 1 does not request Gmail send permission.
- **Actions without approval:** low-risk actions run automatically. An action is low-risk when it stays inside the customer's own account, can be undone, moves no money and contacts no outside party. In Release 1 that means labeling an email and filing a document in Drive. Every QuickBooks write and every outbound message to a third party needs approval.
- **Non-goals:** assumed unchanged. Nothing returns to Release 1 unless you say so.

**Next steps**

1. Review this document and comment on any requirement that is wrong, missing or too strict.
2. Agree the requirements as the Release 1 baseline.
3. Write the architecture document: components, data model, the tool gateway interface, the approval state machine and the security model.
4. Record each major choice as an architecture decision record.
5. Break Release 1 into phases and lessons, each ending with running code and tests.
