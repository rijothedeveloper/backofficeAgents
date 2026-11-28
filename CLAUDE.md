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
- For each lesson, in this order:
  1. Explain the concepts and why this design was chosen.
  2. Show a plan: the files you will create or change and what each does.
     Wait for my go-ahead before writing code.
  3. Write the code in small steps, not one large change. After each step,
     explain it thoroughly: what it does, why it is written this way, and any
     language or framework feature I have not seen before. For Rust, explain
     line by line (ownership, borrowing, lifetimes, traits, async, error handling)
     and compare with how I would do it in Python.
  4. Check my understanding: ask me to explain the key part back, or to make
     one small change myself. Correct me before moving on.
- Never dump a finished lesson in one go. If I say "just do it", still explain after.
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

- Whole stack: `docker compose up --build --wait` (needs `.env`, see `.env.example`),
  stop with `docker compose down`. Health: `localhost:8000/health` (orchestrator),
  `localhost:8080/health` (gateway).
- Orchestrator tests: `cd orchestrator && uv run pytest`
- Gateway tests: `cd gateway && cargo test`
