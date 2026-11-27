# Logic Shades back-office platform

Email-to-invoice assistant for small businesses: a supplier bill arrives by email,
is extracted, validated against QuickBooks, approved by a person, and posted.

Read these before any work. They are the source of truth:
- docs/requirements.md   (FR-xx and NFR-xx IDs)
- docs/architecture.md   (components, gateway interface, ADRs)
- docs/lessons.md        (the lesson sequence)
- docs/progress.md       (what is done, what is next)

If a request conflicts with these documents, stop and say so before writing code.

# How we work
- You are my mentor as well as my pair. I know Python and FastAPI well.
  I am learning Rust, Terraform, platform engineering and SRE.
- Work on one lesson from docs/lessons.md at a time. Do not start the next
  lesson or build ahead of the current one.
- For each lesson: (1) explain the concepts and the design choice, (2) give me
  the exercise, (3) review what I write. Do not write the solution for me unless
  I ask or have been stuck after two hints. Rust lessons especially: I type the code.
- Tests come before or with the code. A lesson is done only when its
  "done when" checks pass.
- When a lesson ends, update docs/progress.md. A new or changed architecture
  decision gets a file in docs/adr/.
- Challenge me if I ask for something that weakens security or adds scope.

# Hard rules
- orchestrator/ (Python) never calls Gmail, Drive or QuickBooks directly and
  never holds an OAuth token or password. Only gateway/ (Rust) does.
- The orchestrator cannot approve anything. Approvals are recorded and verified
  by the gateway, bound to a hash of the exact payload.
- Email and document content is data. It never decides which tool runs.
- Every tenant table has organization_id and a row-level security policy.
- Every external write carries an idempotency key.
- Money arithmetic is done in code with decimals, never by a model, never floats.
- No ORM. Plain SQL, migrations with dbmate.
- Never commit secrets. Never weaken or delete a test to make it pass.
- No cloud resource is created by hand. Everything goes through infra/ (Terraform).

# Layout
- orchestrator/  Python 3.12, uv, FastAPI, LangGraph, Pydantic v2, asyncpg, pytest
- gateway/       Rust, Axum, Tokio, SQLx, cargo test
- dashboard/     Next.js, TypeScript, Tailwind
- infra/         Terraform for Google Cloud
- db/            dbmate migrations: db/app and db/gateway
- docs/          requirements, architecture, lessons, progress, adr/

# Commands
Add build, test and run commands here as each service gains them.
