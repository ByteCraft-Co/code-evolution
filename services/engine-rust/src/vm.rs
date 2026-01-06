use crate::models::genome::{parse_register_index, REGISTER_COUNT};
use crate::models::{Genome, Instruction};

const EPS_DIVISOR: f64 = 1e-12;

pub struct VmConfig {
    pub max_steps: usize,
}

impl Default for VmConfig {
    fn default() -> Self {
        VmConfig { max_steps: 256 }
    }
}

pub enum VmOutcome {
    Ok { output: f64 },
    Invalid { reason: String },
}

pub fn run_genome(genome: &Genome, x: f64, cfg: VmConfig) -> VmOutcome {
    let mut registers = [0.0_f64; REGISTER_COUNT];
    registers[0] = x;
    let mut stack: Vec<f64> = Vec::new();
    let mut pc: usize = 0;
    let mut steps: usize = 0;
    let instructions = &genome.instructions;

    while pc < instructions.len() {
        if steps >= cfg.max_steps {
            return VmOutcome::Invalid {
                reason: "max steps exceeded".to_string(),
            };
        }

        let instr = &instructions[pc];
        steps += 1;

        match instr.op.as_str() {
            "PUSH" => {
                let val = match instr.arg {
                    Some(v) => v,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "PUSH missing arg".to_string(),
                        }
                    }
                };
                stack.push(val);
            }
            "LOAD" => {
                let idx = match parse_register_index(instr.arg) {
                    Ok(i) => i,
                    Err(e) => return VmOutcome::Invalid { reason: e },
                };
                stack.push(registers[idx]);
            }
            "STORE" => {
                let idx = match parse_register_index(instr.arg) {
                    Ok(i) => i,
                    Err(e) => return VmOutcome::Invalid { reason: e },
                };
                let val = match stack.pop() {
                    Some(v) => v,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                registers[idx] = val;
            }
            "ADD" => {
                let (a, b) = match pop_two(&mut stack) {
                    Some(vals) => vals,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                stack.push(a + b);
            }
            "SUB" => {
                let (a, b) = match pop_two(&mut stack) {
                    Some(vals) => vals,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                stack.push(a - b);
            }
            "MUL" => {
                let (a, b) = match pop_two(&mut stack) {
                    Some(vals) => vals,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                stack.push(a * b);
            }
            "DIV" => {
                let (a, b) = match pop_two(&mut stack) {
                    Some(vals) => vals,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                if b.abs() < EPS_DIVISOR {
                    return VmOutcome::Invalid {
                        reason: "division by near-zero".to_string(),
                    };
                }
                stack.push(a / b);
            }
            "DUP" => {
                let top = match stack.last() {
                    Some(v) => *v,
                    None => {
                        return VmOutcome::Invalid {
                            reason: "stack underflow".to_string(),
                        }
                    }
                };
                stack.push(top);
            }
            "SWAP" => {
                if stack.len() < 2 {
                    return VmOutcome::Invalid {
                        reason: "stack underflow".to_string(),
                    };
                }
                let len = stack.len();
                stack.swap(len - 1, len - 2);
            }
            "POP" => {
                if stack.pop().is_none() {
                    return VmOutcome::Invalid {
                        reason: "stack underflow".to_string(),
                    };
                }
            }
            "HALT" => break,
            "NOP" => {}
            _ => {
                return VmOutcome::Invalid {
                    reason: "unknown opcode".to_string(),
                }
            }
        }

        pc += 1;
    }

    let output = stack.last().copied().unwrap_or(registers[0]);
    if !output.is_finite() {
        return VmOutcome::Invalid {
            reason: "non-finite output".to_string(),
        };
    }

    VmOutcome::Ok { output }
}

fn pop_two(stack: &mut Vec<f64>) -> Option<(f64, f64)> {
    if stack.len() < 2 {
        return None;
    }
    let b = stack.pop().unwrap();
    let a = stack.pop().unwrap();
    Some((a, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> VmConfig {
        VmConfig::default()
    }

    fn genome_from_ops(ops: Vec<(&str, Option<f64>)>) -> Genome {
        Genome {
            instructions: ops
                .into_iter()
                .map(|(op, arg)| Instruction {
                    op: op.to_string(),
                    arg,
                })
                .collect(),
        }
    }

    #[test]
    fn adds_numbers() {
        let genome = genome_from_ops(vec![
            ("PUSH", Some(2.0)),
            ("PUSH", Some(3.0)),
            ("ADD", None),
            ("HALT", None),
        ]);
        match run_genome(&genome, 0.0, cfg()) {
            VmOutcome::Ok { output } => assert_eq!(output, 5.0),
            VmOutcome::Invalid { reason } => panic!("unexpected invalid: {reason}"),
        }
    }

    #[test]
    fn multiplies_with_register() {
        let genome = genome_from_ops(vec![
            ("LOAD", Some(0.0)),
            ("PUSH", Some(2.0)),
            ("MUL", None),
            ("HALT", None),
        ]);
        match run_genome(&genome, 4.0, cfg()) {
            VmOutcome::Ok { output } => assert_eq!(output, 8.0),
            VmOutcome::Invalid { reason } => panic!("unexpected invalid: {reason}"),
        }
    }

    #[test]
    fn underflow_is_invalid() {
        let genome = genome_from_ops(vec![("ADD", None)]);
        match run_genome(&genome, 0.0, cfg()) {
            VmOutcome::Ok { output } => panic!("expected invalid, got {output}"),
            VmOutcome::Invalid { .. } => {}
        }
    }

    #[test]
    fn div_by_near_zero_invalid() {
        let genome = genome_from_ops(vec![
            ("PUSH", Some(1.0)),
            ("PUSH", Some(1e-13)),
            ("DIV", None),
        ]);
        match run_genome(&genome, 0.0, cfg()) {
            VmOutcome::Ok { output } => panic!("expected invalid, got {output}"),
            VmOutcome::Invalid { .. } => {}
        }
    }

    #[test]
    fn step_limit_triggers_invalid() {
        let genome = genome_from_ops(vec![("NOP", None), ("NOP", None), ("NOP", None)]);
        let cfg = VmConfig { max_steps: 2 };
        match run_genome(&genome, 0.0, cfg) {
            VmOutcome::Ok { output } => panic!("expected invalid, got {output}"),
            VmOutcome::Invalid { .. } => {}
        }
    }
}
