import {
  RunAdvanceRequest,
  RunConfig,
  RunHistoryResponse,
  RunState,
} from "./types";

const BASE = "/api";

async function handleResponse<T>(res: Response): Promise<T> {
  const text = await res.text();
  const parseJson = () => {
    try {
      return JSON.parse(text);
    } catch {
      return null;
    }
  };
  if (!res.ok) {
    const body = parseJson();
    const message =
      (body && (body.error || body.message)) ||
      text ||
      `Request failed with status ${res.status}`;
    throw new Error(message);
  }
  return (text ? JSON.parse(text) : {}) as T;
}

export async function health() {
  const res = await fetch(`${BASE}/health`);
  return handleResponse<{ status: string }>(res);
}

export async function createRun(cfg: RunConfig) {
  const res = await fetch(`${BASE}/runs`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(cfg),
  });
  return handleResponse<{ run_id: string }>(res);
}

export async function getRun(run_id: string): Promise<RunState> {
  const res = await fetch(`${BASE}/runs/${run_id}`);
  return handleResponse<RunState>(res);
}

export async function stepRun(run_id: string): Promise<RunState> {
  const res = await fetch(`${BASE}/runs/${run_id}/step`, { method: "POST" });
  return handleResponse<RunState>(res);
}

export async function advanceRun(
  run_id: string,
  steps: number,
): Promise<RunState> {
  const body: RunAdvanceRequest = { steps };
  const res = await fetch(`${BASE}/runs/${run_id}/advance`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return handleResponse<RunState>(res);
}

export async function getHistory(run_id: string): Promise<RunHistoryResponse> {
  const res = await fetch(`${BASE}/runs/${run_id}/history`);
  return handleResponse<RunHistoryResponse>(res);
}
