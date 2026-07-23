from fastapi import FastAPI

app = FastAPI()


@app.get("/health")
def health() -> dict:
    return {"ok": True}


@app.post("/items")
def create_item() -> dict:
    return {"id": 1}


class Item:
    def __init__(self, name: str) -> None:
        self.name = name
