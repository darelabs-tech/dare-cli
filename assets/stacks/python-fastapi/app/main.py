from fastapi import FastAPI

app = FastAPI(title="{{project_name}}")


@app.get("/healthz")
def healthz() -> dict[str, str]:
    return {"status": "ok"}
