use std::collections::HashMap;
use std::sync::Arc;

use rand::{rngs::StdRng, Rng};
use tokio::sync::Mutex;

use crate::error::EngineError;
use crate::models::{genome, rng, Genome, RunConfig, RunState};

pub type RunStore = Arc<Mutex<HashMap<String, RunInternal>>>;

pub struct RunInternal {
    pub cfg: RunConfig,
    pub generation: u32,
    pub population: Vec<Genome>,
    pub fitness: Vec<f64>,
    pub best_fitness: f64,
    pub best_genome: Genome,
    pub rng: StdRng,
    pub history: Vec<(u32, f64)>,
}

impl RunInternal {
    pub fn new(cfg: RunConfig) -> Self {
        let size = usize::try_from(cfg.population.max(1)).unwrap_or(1);
        let mut rng = rng::seeded_rng(cfg.seed as u64);
        let population = (0..size).map(|_| genome::random_genome(&mut rng)).collect();
        Self {
            cfg,
            generation: 0,
            population,
            fitness: Vec::new(),
            best_fitness: 0.0,
            best_genome: Genome {
                instructions: Vec::new(),
            },
            rng,
            history: Vec::new(),
        }
    }

    pub fn to_state(&self, run_id: &str) -> RunState {
        RunState {
            run_id: run_id.to_string(),
            generation: self.generation as i64,
            best_fitness: self.best_fitness,
            best_genome: self.best_genome.clone(),
            seed: self.cfg.seed as u64,
            population: self.cfg.population as u32,
            generations: self.cfg.generations as u32,
            mutation_rate: self.cfg.mutation_rate as f32,
            task: self.cfg.task.clone(),
        }
    }

    pub fn apply_fitness(&mut self, fitness: Vec<f64>) {
        self.fitness = fitness;
        if let Some((idx, best)) = best_index(&self.fitness) {
            self.best_fitness = best;
            self.best_genome = self.population[idx].clone();
        }
        self.history.push((self.generation, self.best_fitness));
    }

    pub fn next_population(&mut self) -> Vec<Genome> {
        let pop_size = self.population.len();
        let mut new_pop = Vec::with_capacity(pop_size);
        // Elitism
        new_pop.push(self.best_genome.clone());

        while new_pop.len() < pop_size {
            let parent_idx = self.tournament_select(3);
            let mut child = self.population[parent_idx].clone();
            if self.rng.gen::<f64>() < self.cfg.mutation_rate {
                genome::mutate_genome(&mut child, &mut self.rng);
            }
            new_pop.push(child);
        }

        new_pop
    }

    fn tournament_select(&mut self, k: usize) -> usize {
        let mut best_idx = 0;
        let mut best_fit = f64::MIN;
        let len = self.population.len();
        for _ in 0..k {
            let idx = self.rng.gen_range(0..len);
            let fit = *self.fitness.get(idx).unwrap_or(&0.0);
            if fit > best_fit {
                best_fit = fit;
                best_idx = idx;
            }
        }
        best_idx
    }
}

pub fn new_store() -> RunStore {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn create_run(
    cfg: RunConfig,
    runs: &RunStore,
    fitness_url: &str,
) -> Result<String, EngineError> {
    let mut run = RunInternal::new(cfg);
    let scores = score_population(&run.cfg.task, &run.population, fitness_url).await?;
    run.apply_fitness(scores);
    let run_id = generate_run_id(&mut run.rng);
    let task = run.cfg.task.clone();
    let pop = run.cfg.population;

    let mut guard = runs.lock().await;
    guard.insert(run_id.clone(), run);
    tracing::info!("run created id={} task={} pop={}", run_id, task, pop);
    Ok(run_id)
}

pub async fn get_run_state(runs: &RunStore, run_id: &str) -> Option<RunState> {
    let guard = runs.lock().await;
    guard.get(run_id).map(|r| r.to_state(run_id))
}

pub async fn step_run(
    runs: &RunStore,
    run_id: &str,
    fitness_url: &str,
) -> Result<RunState, EngineError> {
    let (new_population, cfg_task, pop_size) = {
        let mut guard = runs.lock().await;
        let run = guard
            .get_mut(run_id)
            .ok_or_else(|| EngineError::NotFound("run not found".to_string()))?;
        let new_pop = run.next_population();
        let task = run.cfg.task.clone();
        (new_pop, task, run.population.len())
    };

    let scores = score_population(&cfg_task, &new_population, fitness_url).await?;

    let mut guard = runs.lock().await;
    let run = guard
        .get_mut(run_id)
        .ok_or_else(|| EngineError::NotFound("run not found".to_string()))?;
    if new_population.len() != pop_size {
        return Err(EngineError::InternalError(
            "population size mismatch".to_string(),
        ));
    }
    run.population = new_population;
    run.generation += 1;
    run.apply_fitness(scores);
    tracing::info!(
        "generation step run_id={} gen={} best_fitness={}",
        run_id,
        run.generation,
        run.best_fitness
    );

    Ok(run.to_state(run_id))
}

pub async fn advance_run(
    runs: &RunStore,
    run_id: &str,
    steps: u32,
    fitness_url: &str,
) -> Result<RunState, EngineError> {
    let mut last_state = None;
    for _ in 0..steps {
        last_state = Some(step_run(runs, run_id, fitness_url).await?);
    }
    last_state.ok_or_else(|| EngineError::InternalError("no steps executed".to_string()))
}

pub async fn get_history(
    runs: &RunStore,
    run_id: &str,
) -> Option<Vec<(u32, f64)>> {
    let guard = runs.lock().await;
    guard.get(run_id).map(|r| r.history.clone())
}

pub async fn score_population(
    task: &str,
    genomes: &[Genome],
    fitness_url: &str,
) -> Result<Vec<f64>, EngineError> {
    #[derive(serde::Deserialize)]
    struct ScoreResponse {
        fitness: Vec<f64>,
    }

    let url = format!("{}/score", fitness_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&serde_json::json!({ "task": task, "genomes": genomes }))
        .send()
        .await
        .map_err(|e| EngineError::InternalError(format!("fitness request failed: {e}")))?;

    let resp = resp
        .error_for_status()
        .map_err(|e| EngineError::InternalError(format!("fitness status error: {e}")))?;

    let body: ScoreResponse = resp
        .json()
        .await
        .map_err(|e| EngineError::InternalError(format!("fitness decode failed: {e}")))?;

    Ok(body.fitness)
}

fn generate_run_id(rng: &mut StdRng) -> String {
    let v: u64 = rng.gen();
    format!("{:016x}", v)
}

fn best_index(fitness: &[f64]) -> Option<(usize, f64)> {
    fitness
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, &f)| (i, f))
}
