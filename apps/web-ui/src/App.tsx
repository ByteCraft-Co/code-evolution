import { useEffect, useMemo, useState } from "react";
import {
  advanceRun,
  createRun,
  getHistory,
  getRun,
  stepRun,
} from "./api";
import {
  Instruction,
  RunConfig,
  RunHistoryPoint,
  RunHistoryResponse,
  RunState,
} from "./types";

type FormState = {
  seed: number;
  population: number;
  generations: number;
  mutation_rate: number;
  task: string;
};

const defaultForm: FormState = {
  seed: 1,
  population: 50,
  generations: 200,
  mutation_rate: 0.25,
  task: "poly2",
};

export default function App() {
  const [form, setForm] = useState<FormState>(defaultForm);
  const [runId, setRunId] = useState<string>("");
  const [runState, setRunState] = useState<RunState | null>(null);
  const [history, setHistory] = useState<RunHistoryPoint[]>([]);
  const [advanceSteps, setAdvanceSteps] = useState<number>(10);
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>("");

  const sortedHistory = useMemo(() => {
    if (!history || history.length === 0) return [];
    return [...history].sort((a, b) => a.generation - b.generation);
  }, [history]);

  const lastHistory = useMemo(() => {
    const maxPoints = 30;
    if (sortedHistory.length <= maxPoints) return sortedHistory;
    return sortedHistory.slice(-maxPoints);
  }, [sortedHistory]);

  const handleChange = (field: keyof FormState, value: string) => {
    setForm((prev) => ({
      ...prev,
      [field]: field === "task" ? value : Number(value),
    }));
  };

  const handleStart = async () => {
    setError("");
    setLoading(true);
    try {
      const cfg: RunConfig = { ...form };
      const res = await createRun(cfg);
      setRunId(res.run_id);
      await refreshState(res.run_id);
      await loadHistory(res.run_id);
    } catch (e: any) {
      setError(e?.message || "Failed to start run");
    } finally {
      setLoading(false);
    }
  };

  const refreshState = async (id: string) => {
    const state = await getRun(id);
    setRunState(state);
  };

  const loadHistory = async (id: string) => {
    const h: RunHistoryResponse = await getHistory(id);
    setHistory(h.points || []);
  };

  const handleRefresh = async () => {
    if (!runId) return;
    setError("");
    setLoading(true);
    try {
      await refreshState(runId);
      await loadHistory(runId);
    } catch (e: any) {
      setError(e?.message || "Failed to refresh");
    } finally {
      setLoading(false);
    }
  };

  const handleStep = async () => {
    if (!runId) return;
    setError("");
    setLoading(true);
    try {
      const updated = await stepRun(runId);
      setRunState(updated);
      await loadHistory(runId);
    } catch (e: any) {
      setError(e?.message || "Failed to step");
    } finally {
      setLoading(false);
    }
  };

  const handleAdvance = async () => {
    if (!runId) return;
    setError("");
    setLoading(true);
    try {
      const updated = await advanceRun(runId, advanceSteps);
      setRunState(updated);
      await loadHistory(runId);
    } catch (e: any) {
      setError(e?.message || "Failed to advance");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    setError("");
  }, [runId]);

  return (
    <div style={styles.app}>
      <h1 style={styles.heading}>Code Evolution Control</h1>

      <section style={styles.section}>
        <h2 style={styles.subheading}>Start Run</h2>
        <div style={styles.formGrid}>
          <label style={styles.label}>
            Seed
            <input
              type="number"
              value={form.seed}
              onChange={(e) => handleChange("seed", e.target.value)}
              style={styles.input}
            />
          </label>
          <label style={styles.label}>
            Population
            <input
              type="number"
              value={form.population}
              onChange={(e) => handleChange("population", e.target.value)}
              style={styles.input}
            />
          </label>
          <label style={styles.label}>
            Generations
            <input
              type="number"
              value={form.generations}
              onChange={(e) => handleChange("generations", e.target.value)}
              style={styles.input}
            />
          </label>
          <label style={styles.label}>
            Mutation Rate
            <input
              type="number"
              step="0.01"
              value={form.mutation_rate}
              onChange={(e) => handleChange("mutation_rate", e.target.value)}
              style={styles.input}
            />
          </label>
          <label style={styles.label}>
            Task
            <input
              type="text"
              value={form.task}
              onChange={(e) => handleChange("task", e.target.value)}
              style={styles.input}
            />
          </label>
        </div>
        <button
          style={styles.button}
          onClick={handleStart}
          disabled={loading}
        >
          {loading ? "Working..." : "Start Run"}
        </button>
      </section>

      {error ? <div style={styles.error}>Error: {error}</div> : null}

      {runId ? (
        <section style={styles.section}>
          <h2 style={styles.subheading}>Run Controls</h2>
          <div style={styles.controls}>
            <button
              style={styles.button}
              onClick={handleRefresh}
              disabled={loading}
            >
              Refresh
            </button>
            <button
              style={styles.button}
              onClick={handleStep}
              disabled={loading}
            >
              Step
            </button>
            <div style={styles.inline}>
              <input
                type="number"
                value={advanceSteps}
                min={1}
                onChange={(e) => setAdvanceSteps(Number(e.target.value))}
                style={{ ...styles.input, width: "100px" }}
              />
              <button
                style={styles.button}
                onClick={handleAdvance}
                disabled={loading}
              >
                Advance
              </button>
            </div>
            <button
              style={styles.button}
              onClick={() => loadHistory(runId)}
              disabled={loading}
            >
              Load History
            </button>
          </div>

          {runState ? (
            <div style={styles.panel}>
              <div>Run ID: {runState.run_id}</div>
              <div>Generation: {runState.generation}</div>
              <div>
                Best fitness: {runState.best_fitness.toFixed(6)}
              </div>
              <div style={{ marginTop: "8px" }}>
                <strong>Best genome:</strong>
                {renderGenome(runState.best_genome.instructions)}
              </div>
            </div>
          ) : (
            <div style={styles.panel}>No run state loaded.</div>
          )}

          <div style={styles.panel}>
            <h3 style={styles.subheading}>History</h3>
            {lastHistory.length === 0 ? (
              <div>No history yet.</div>
            ) : (
              <ul style={styles.list}>
                {lastHistory.map((p, idx) => (
                  <li key={`${p.generation}-${idx}`}>
                    gen: {p.generation} | fitness: {p.best_fitness.toFixed(6)}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </section>
      ) : null}
    </div>
  );
}

function renderGenome(instructions: Instruction[]) {
  if (!instructions || instructions.length === 0) {
    return <div>Empty genome</div>;
  }
  return (
    <pre style={styles.code}>
      {instructions.map((instr, idx) => {
        const arg = instr.arg === null ? "" : ` ${instr.arg.toFixed(4)}`;
        return `${idx}: ${instr.op}${arg}`;
      }).join("\n")}
    </pre>
  );
}

const styles: Record<string, React.CSSProperties> = {
  app: {
    fontFamily: "Arial, sans-serif",
    color: "#222",
    padding: "20px",
    lineHeight: 1.4,
    maxWidth: "960px",
    margin: "0 auto",
  },
  heading: {
    marginBottom: "12px",
  },
  subheading: {
    margin: "8px 0",
  },
  section: {
    border: "1px solid #ddd",
    borderRadius: "6px",
    padding: "12px",
    marginBottom: "16px",
    background: "#fafafa",
  },
  formGrid: {
    display: "grid",
    gridTemplateColumns: "repeat(auto-fit, minmax(160px, 1fr))",
    gap: "8px",
    marginBottom: "12px",
  },
  label: {
    display: "flex",
    flexDirection: "column",
    gap: "4px",
    fontSize: "0.9rem",
  },
  input: {
    padding: "6px",
    border: "1px solid #ccc",
    borderRadius: "4px",
    fontSize: "1rem",
  },
  button: {
    padding: "8px 12px",
    marginRight: "8px",
    border: "none",
    borderRadius: "4px",
    background: "#2563eb",
    color: "#fff",
    cursor: "pointer",
  },
  controls: {
    display: "flex",
    flexWrap: "wrap",
    gap: "8px",
    alignItems: "center",
    marginBottom: "12px",
  },
  inline: {
    display: "flex",
    gap: "6px",
    alignItems: "center",
  },
  panel: {
    border: "1px solid #e5e7eb",
    borderRadius: "6px",
    padding: "10px",
    marginBottom: "10px",
    background: "#fff",
  },
  code: {
    background: "#f5f5f5",
    padding: "8px",
    borderRadius: "4px",
    whiteSpace: "pre-wrap",
    marginTop: "6px",
  },
  list: {
    listStyle: "none",
    padding: 0,
    margin: 0,
  },
  error: {
    background: "#fee2e2",
    color: "#b91c1c",
    padding: "8px",
    borderRadius: "4px",
    margin: "8px 0",
  },
};
