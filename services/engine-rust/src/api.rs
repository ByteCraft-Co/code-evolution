use std::env;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::json;

use crate::error::EngineError;
use crate::models::evolve::{
    advance_run, create_run, get_history, get_run_state, new_store, step_run, RunStore,
};
use crate::models::{
    RunAdvanceRequest, RunConfig, RunHistoryPoint, RunHistoryResponse, RunState,
};

#[derive(Clone)]
struct AppState {
    runs: RunStore,
    fitness_url: String,
}

pub fn router() -> Router {
    let fitness_url =
        env::var("FITNESS_URL").unwrap_or_else(|_| "http://127.0.0.1:8090".to_string());
    let state = AppState {
        runs: new_store(),
        fitness_url,
    };

    Router::new()
        .route("/health", get(health))
        .route("/runs", post(create_run_handler))
        .route("/runs/:run_id", get(get_run))
        .route("/runs/:run_id/step", post(step_run_handler))
        .route("/runs/:run_id/history", get(get_history_handler))
        .route("/runs/:run_id/advance", post(advance_run_handler))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

async fn create_run_handler(
    State(state): State<AppState>,
    Json(cfg): Json<RunConfig>,
) -> Result<Json<serde_json::Value>, EngineError> {
    validate_run_config(&cfg)?;
    let run_id = create_run(cfg, &state.runs, &state.fitness_url).await?;
    tracing::info!("created run {}", run_id);
    Ok(Json(json!({ "run_id": run_id })))
}

async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunState>, EngineError> {
    if let Some(run) = get_run_state(&state.runs, &run_id).await {
        Ok(Json(run))
    } else {
        Err(EngineError::NotFound("run not found".to_string()))
    }
}

async fn step_run_handler(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunState>, EngineError> {
    let updated = step_run(&state.runs, &run_id, &state.fitness_url).await?;
    tracing::info!("stepped run {} to generation {}", run_id, updated.generation);
    Ok(Json(updated))
}

async fn get_history_handler(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
) -> Result<Json<RunHistoryResponse>, EngineError> {
    let maybe_points = get_history(&state.runs, &run_id).await;
    if let Some(points) = maybe_points {
        let run_state = get_run_state(&state.runs, &run_id)
            .await
            .ok_or_else(|| EngineError::NotFound("run not found".to_string()))?;
        let points = points
            .into_iter()
            .map(|(generation, best_fitness)| RunHistoryPoint {
                generation,
                best_fitness,
            })
            .collect();
        Ok(Json(RunHistoryResponse {
            run_id,
            task: run_state.task.clone(),
            points,
        }))
    } else {
        Err(EngineError::NotFound("run not found".to_string()))
    }
}

async fn advance_run_handler(
    State(state): State<AppState>,
    Path(run_id): Path<String>,
    Json(req): Json<RunAdvanceRequest>,
) -> Result<Json<RunState>, EngineError> {
    validate_advance(&req)?;
    let updated = advance_run(&state.runs, &run_id, req.steps, &state.fitness_url).await?;
    tracing::info!(
        "advance run {} steps={} final_gen={} best={}",
        run_id,
        req.steps,
        updated.generation,
        updated.best_fitness
    );
    Ok(Json(updated))
}

fn validate_run_config(cfg: &RunConfig) -> Result<(), EngineError> {
    if !(1..=5000).contains(&cfg.population) {
        return Err(EngineError::BadRequest("population out of range".to_string()));
    }
    if !(1..=1_000_000).contains(&cfg.generations) {
        return Err(EngineError::BadRequest("generations out of range".to_string()));
    }
    if !(0.0..=1.0).contains(&cfg.mutation_rate) {
        return Err(EngineError::BadRequest("mutation_rate must be between 0 and 1".to_string()));
    }
    if cfg.task.trim().is_empty() {
        return Err(EngineError::BadRequest("task must be non-empty".to_string()));
    }
    Ok(())
}

fn validate_advance(req: &RunAdvanceRequest) -> Result<(), EngineError> {
    if !(1..=10_000).contains(&req.steps) {
        return Err(EngineError::BadRequest(
            "steps must be between 1 and 10000".to_string(),
        ));
    }
    Ok(())
}
