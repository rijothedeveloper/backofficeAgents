from fastapi import FastAPI

app = FastAPI(title="Logic Shades orchestrator")


@app.get("/health")
async def health() -> dict[str, str]:
    return {"status": "ok"}
