use serde::{Deserialize, Serialize};

#[path = "genome.rs"]
pub mod genome;
#[path = "vm.rs"]
pub mod vm;
#[path = "rng.rs"]
pub mod rng;
#[path = "evolve.rs"]
pub mod evolve;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub op: String,
    pub arg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunConfig {
    pub seed: i64,
    pub population: i64,
    pub generations: i64,
    pub mutation_rate: f64,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: String,
    pub generation: i64,
    pub best_fitness: f64,
    pub best_genome: Genome,
    pub seed: u64,
    pub population: u32,
    pub generations: u32,
    pub mutation_rate: f32,
    pub task: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHistoryPoint {
    pub generation: u32,
    pub best_fitness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunHistoryResponse {
    pub run_id: String,
    pub task: String,
    pub points: Vec<RunHistoryPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunAdvanceRequest {
    pub steps: u32,
}
