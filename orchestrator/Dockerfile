FROM python:3.12-slim

COPY --from=ghcr.io/astral-sh/uv:0.13.9 /uv /usr/local/bin/uv

WORKDIR /app

# Dependencies first, so this layer is reused until the lockfile changes.
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project

COPY README.md ./
COPY src ./src
RUN uv sync --frozen --no-dev

RUN useradd --system --uid 10001 --no-create-home app
USER app

ENV PATH="/app/.venv/bin:$PATH"
EXPOSE 8000
CMD ["uvicorn", "orchestrator.main:app", "--host", "0.0.0.0", "--port", "8000"]