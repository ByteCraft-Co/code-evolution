from __future__ import annotations

import math
from typing import List, Tuple

from .models import Genome
from .tasks import get_cases

EPS_DIVISOR = 1e-12
DEFAULT_MAX_STEPS = 256


def _parse_register(arg: float | None) -> int | None:
    if arg in (0.0, 1.0, 2.0, 3.0):
        return int(arg)
    return None


def _pop_two(stack: list[float]) -> tuple[float, float] | None:
    if len(stack) < 2:
        return None
    b = stack.pop()
    a = stack.pop()
    return a, b


def run_genome(genome: Genome, x: float, max_steps: int = DEFAULT_MAX_STEPS) -> tuple[bool, float | None]:
    registers: List[float] = [0.0, 0.0, 0.0, 0.0]
    registers[0] = x
    stack: List[float] = []
    instructions = genome.instructions
    pc = 0
    steps = 0

    while pc < len(instructions):
        if steps >= max_steps:
            return False, None

        instr = instructions[pc]
        steps += 1
        op = instr.op

        if op == "PUSH":
            if instr.arg is None:
                return False, None
            stack.append(instr.arg)
        elif op == "LOAD":
            idx = _parse_register(instr.arg)
            if idx is None:
                return False, None
            stack.append(registers[idx])
        elif op == "STORE":
            idx = _parse_register(instr.arg)
            if idx is None or not stack:
                return False, None
            registers[idx] = stack.pop()
        elif op == "ADD":
            values = _pop_two(stack)
            if values is None:
                return False, None
            a, b = values
            stack.append(a + b)
        elif op == "SUB":
            values = _pop_two(stack)
            if values is None:
                return False, None
            a, b = values
            stack.append(a - b)
        elif op == "MUL":
            values = _pop_two(stack)
            if values is None:
                return False, None
            a, b = values
            stack.append(a * b)
        elif op == "DIV":
            values = _pop_two(stack)
            if values is None:
                return False, None
            a, b = values
            if abs(b) < EPS_DIVISOR:
                return False, None
            stack.append(a / b)
        elif op == "DUP":
            if not stack:
                return False, None
            stack.append(stack[-1])
        elif op == "SWAP":
            if len(stack) < 2:
                return False, None
            stack[-1], stack[-2] = stack[-2], stack[-1]
        elif op == "POP":
            if not stack:
                return False, None
            stack.pop()
        elif op == "HALT":
            break
        elif op == "NOP":
            pass
        else:
            return False, None

        pc += 1

    output = stack[-1] if stack else registers[0]
    if not math.isfinite(output):
        return False, None

    return True, output


def score_genomes(task: str, genomes: List[Genome]) -> List[float]:
    cases: List[Tuple[float, float]] = get_cases(task)
    fitnesses: List[float] = []
    for genome in genomes:
        invalid = False
        errors: List[float] = []
        for x, expected in cases:
            ok, output = run_genome(genome, x)
            if not ok or output is None:
                invalid = True
                break
            errors.append(abs(output - expected))
        if invalid:
            fitnesses.append(1e-9)
        else:
            mean_error = sum(errors) / len(errors) if errors else float("inf")
            fitnesses.append(1.0 / (1.0 + mean_error))
    return fitnesses
