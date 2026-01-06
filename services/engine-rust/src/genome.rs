use rand::{rngs::StdRng, Rng};

use crate::models::rng::{gen_range_f64, gen_range_usize};
use crate::models::{Genome, Instruction};

pub const REGISTER_COUNT: usize = 4;
const OPS: [&str; 12] = [
    "PUSH", "LOAD", "STORE", "ADD", "SUB", "MUL", "DIV", "DUP", "SWAP", "POP", "HALT", "NOP",
];
const MIN_LEN: usize = 8;
const MAX_LEN: usize = 32;
const ABS_MAX_LEN: usize = 64;

pub fn parse_register_index(arg: Option<f64>) -> Result<usize, String> {
    match arg {
        Some(val) if val == 0.0 || val == 1.0 || val == 2.0 || val == 3.0 => {
            Ok(val as usize)
        }
        _ => Err("invalid register index".to_string()),
    }
}

pub fn random_instruction(rng: &mut StdRng) -> Instruction {
    let op = OPS[gen_range_usize(rng, OPS.len())];
    match op {
        "PUSH" => Instruction {
            op: op.to_string(),
            arg: Some(gen_range_f64(rng, -10.0, 10.0)),
        },
        "LOAD" | "STORE" => Instruction {
            op: op.to_string(),
            arg: Some(gen_range_usize(rng, REGISTER_COUNT) as f64),
        },
        _ => Instruction {
            op: op.to_string(),
            arg: None,
        },
    }
}

pub fn random_genome(rng: &mut StdRng) -> Genome {
    let len = rng.gen_range(MIN_LEN..=MAX_LEN);
    Genome {
        instructions: (0..len).map(|_| random_instruction(rng)).collect(),
    }
}

pub fn mutate_genome(genome: &mut Genome, rng: &mut StdRng) {
    if genome.instructions.is_empty() {
        genome.instructions.push(random_instruction(rng));
        return;
    }

    let choice = rng.gen_range(0..4);
    match choice {
        0 => point_mutate(genome, rng),
        1 => tweak_push(genome, rng),
        2 => insert_instruction(genome, rng),
        _ => delete_instruction(genome, rng),
    }

    if genome.instructions.is_empty() {
        genome.instructions.push(random_instruction(rng));
    }
    if genome.instructions.len() > ABS_MAX_LEN {
        genome.instructions.truncate(ABS_MAX_LEN);
    }
}

fn point_mutate(genome: &mut Genome, rng: &mut StdRng) {
    let idx = gen_range_usize(rng, genome.instructions.len());
    genome.instructions[idx] = random_instruction(rng);
}

fn tweak_push(genome: &mut Genome, rng: &mut StdRng) {
    let push_indices: Vec<usize> = genome
        .instructions
        .iter()
        .enumerate()
        .filter_map(|(i, instr)| (instr.op == "PUSH").then_some(i))
        .collect();
    if push_indices.is_empty() {
        point_mutate(genome, rng);
        return;
    }
    let idx = push_indices[rng.gen_range(0..push_indices.len())];
    let noise = gen_range_f64(rng, -1.0, 1.0);
    let new_val = genome.instructions[idx].arg.unwrap_or(0.0) + noise;
    genome.instructions[idx].arg = Some(new_val.clamp(-10.0, 10.0));
}

fn insert_instruction(genome: &mut Genome, rng: &mut StdRng) {
    if genome.instructions.len() >= ABS_MAX_LEN {
        point_mutate(genome, rng);
        return;
    }
    let idx = rng.gen_range(0..=genome.instructions.len());
    genome.instructions.insert(idx, random_instruction(rng));
}

fn delete_instruction(genome: &mut Genome, rng: &mut StdRng) {
    if genome.instructions.len() <= 1 {
        return;
    }
    let idx = gen_range_usize(rng, genome.instructions.len());
    genome.instructions.remove(idx);
}
