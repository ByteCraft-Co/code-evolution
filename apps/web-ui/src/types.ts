export type Instruction = {
  op: string;
  arg: number | null;
};

export type Genome = {
  instructions: Instruction[];
};

export type RunConfig = {
  seed: number;
  population: number;
  generations: number;
  mutation_rate: number;
  task: string;
};

export type RunState = {
  run_id: string;
  generation: number;
  best_fitness: number;
  best_genome: Genome;
  seed?: number;
  population?: number;
  generations?: number;
  mutation_rate?: number;
  task?: string;
};

export type RunHistoryPoint = {
  generation: number;
  best_fitness: number;
};

export type RunHistoryResponse = {
  run_id: string;
  task: string;
  points: RunHistoryPoint[];
};

export type RunAdvanceRequest = {
  steps: number;
};
