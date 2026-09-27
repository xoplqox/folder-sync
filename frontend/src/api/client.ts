import type { DrivesResponse } from "../types/drive";
import type { ComparisonMode, ConfigResponse, ConfigUpdate } from "../types/config";
import type { MergedTree } from "../types/tree";
import type { BatchRun, QueueRequest, QueueResponseBody } from "../types/batch";

class ApiError extends Error {}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, init);
  if (!res.ok) {
    let message = res.statusText;
    try {
      const body = (await res.json()) as { error?: string };
      if (body?.error) message = body.error;
    } catch {
      // response body wasn't JSON; fall back to statusText
    }
    throw new ApiError(message);
  }
  if (res.status === 204 || res.headers.get("content-length") === "0") {
    return undefined as T;
  }
  const text = await res.text();
  return (text ? JSON.parse(text) : undefined) as T;
}

export function getDrives(): Promise<DrivesResponse> {
  return request<DrivesResponse>("/api/drives");
}

export function rescanDrives(): Promise<DrivesResponse> {
  return request<DrivesResponse>("/api/drives/rescan", { method: "POST" });
}

export function getTree(
  name: string,
  number: string,
  mode?: ComparisonMode,
): Promise<MergedTree> {
  const query = mode ? `?mode=${encodeURIComponent(mode)}` : "";
  return request<MergedTree>(
    `/api/drives/${encodeURIComponent(name)}/${encodeURIComponent(number)}/tree${query}`,
  );
}

export function getConfig(): Promise<ConfigResponse> {
  return request<ConfigResponse>("/api/config");
}

export function updateConfig(update: ConfigUpdate): Promise<ConfigResponse> {
  return request<ConfigResponse>("/api/config", {
    method: "PUT",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(update),
  });
}

export function getBatch(): Promise<BatchRun> {
  return request<BatchRun>("/api/batch");
}

export function queueBatchAction(req: QueueRequest): Promise<QueueResponseBody> {
  return request<QueueResponseBody>("/api/batch/actions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(req),
  });
}

export function removeBatchAction(id: string): Promise<BatchRun> {
  return request<BatchRun>(`/api/batch/actions/${encodeURIComponent(id)}`, {
    method: "DELETE",
  });
}

export function removeBatchGroup(groupId: string): Promise<BatchRun> {
  return request<BatchRun>(`/api/batch/actions?group_id=${encodeURIComponent(groupId)}`, {
    method: "DELETE",
  });
}

export function startBatch(): Promise<void> {
  return request<void>("/api/batch/start", { method: "POST" });
}

export function cancelBatch(): Promise<void> {
  return request<void>("/api/batch/cancel", { method: "POST" });
}

export { ApiError };
