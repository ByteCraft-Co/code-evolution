from typing import List, Optional

from pydantic import BaseModel


class Instruction(BaseModel):
    op: str
    arg: Optional[float] = None


class Genome(BaseModel):
    instructions: List[Instruction]


class ScoreRequest(BaseModel):
    task: str
    genomes: List[Genome]


class ScoreResponse(BaseModel):
    fitness: List[float]
