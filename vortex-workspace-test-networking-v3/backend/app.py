#!/usr/bin/env python3
# Simple Python backend test
from fastapi import FastAPI
import uvicorn

app = FastAPI()

@app.get("/test")
def test_endpoint():
    return "Hello from backend! Networking is working!"

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8000)