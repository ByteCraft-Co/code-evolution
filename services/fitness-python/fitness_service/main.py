from fastapi import FastAPI

from .models import ScoreRequest, ScoreResponse
from .scoring import score_genomes

app = FastAPI(title="Fitness Service")


@app.get("/health")
async def health() -> dict:
    return {"status": "ok"}


@app.post("/score", response_model=ScoreResponse)
async def score(request: ScoreRequest) -> ScoreResponse:
    fitness = score_genomes(request.task, request.genomes)
    return ScoreResponse(fitness=fitness)
