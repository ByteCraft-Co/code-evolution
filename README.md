# Code Evolution Simulator

## Overview
Code Evolution Simulator explores evolutionary computation: small programs (“genomes”) evolve over generations to solve tasks. Populations mutate, compete, and are selected based on fitness scores. This is evolutionary computation, not machine learning; behavior emerges from variation, selection, and scoring rather than gradient-based training.

## High-Level Architecture
- **Rust Engine (`services/engine-rust`)**  
  Runs the evolution loop, manages populations and generations, calls the fitness service, and exposes an HTTP API.
- **Python Fitness Service (`services/fitness-python`)**  
  Evaluates genomes on tasks and computes fitness scores; simple to extend with new tasks.
- **Web UI (`apps/web-ui`)**  
  Controls the engine, starts runs, steps or advances generations, and displays the best genome and fitness history.

Services communicate via HTTP using a shared schema (`shared/schema/openapi.yaml`).

## Genome & VM Model (Option A)
A genome is a list of instructions executed by a stack-machine VM:
- Stack plus four registers (r0..r3); input `x` is placed into `r0`.
- Instruction set: `PUSH`, `LOAD`, `STORE`, `ADD`, `SUB`, `MUL`, `DIV`, `DUP`, `SWAP`, `POP`, `HALT`, `NOP`.
- Output: top of stack if non-empty, else `r0`.
- Invalid if: stack underflow, bad register index, division by near-zero, step limit exceeded, or non-finite output.

Example genome:
```
0: LOAD 0
1: PUSH 2.0
2: MUL
3: HALT
```

## Evolution Process
- Initialize a random population.
- Score genomes via the Python fitness service.
- Tournament selection (k=3) with elitism (best carries over).
- Mutation operators: point mutate, tweak a `PUSH` constant, insert, or delete instructions.
- Deterministic behavior via seeded RNG in the engine.

## API Overview
- `POST /runs` — start a run
- `POST /runs/{id}/step` — advance one generation
- `POST /runs/{id}/advance` — advance multiple generations
- `GET /runs/{id}` — fetch run state
- `GET /runs/{id}/history` — fetch fitness history

OpenAPI spec: `shared/schema/openapi.yaml`.

## Running the Project (Local Development)
Requirements: Rust toolchain, Python 3.10+, Poetry, Node.js + npm.

1) Fitness Service  
```
cd services/fitness-python
poetry install
poetry run uvicorn fitness_service.main:app --reload --port 8090
```
2) Rust Engine  
```
cd services/engine-rust
cargo run
```
3) Web UI  
```
cd apps/web-ui
npm install
npm run dev
```

Endpoints:
- Engine: http://127.0.0.1:8080
- Fitness: http://127.0.0.1:8090
- UI: http://127.0.0.1:5173 (Vite proxies `/api` to the engine to avoid CORS)

## Example Workflow
Start a run, then step or advance generations, watch fitness improve, and inspect the best genome in the UI or via the API.

## Project Status
Core system complete; UI complete; runs are in-memory by design. Built as a learning tool for multi-language system architecture.

## Possible Extensions
- Additional tasks (sorting, maze solving)
- Persistence with a database
- Charts/visualizations
- WASM execution
- Performance optimizations
