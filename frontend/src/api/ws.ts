import { useBatchStore } from "../stores/batch";
import type { BatchRun } from "../types/batch";

export type ProgressEvent =
  | { type: "snapshot" }
  | { type: "run_started" }
  | { type: "action_started"; action_id: string }
  | { type: "action_progress"; action_id: string; bytes_done: number; bytes_total: number }
  | { type: "action_finished"; action_id: string; status: string; error: string | null }
  | { type: "run_cancelled" }
  | { type: "run_completed"; succeeded: number; failed: number };

interface WsMessage {
  event: ProgressEvent;
  run: BatchRun;
}

let socket: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

/** Opens (or reuses) the single WebSocket connection for live batch progress. Auto-reconnects on drop. */
export function connectBatchEvents(): void {
  if (socket) return;

  const protocol = window.location.protocol === "https:" ? "wss" : "ws";
  const url = `${protocol}://${window.location.host}/api/events`;

  const open = () => {
    const batch = useBatchStore();
    const ws = new WebSocket(url);
    socket = ws;

    ws.addEventListener("message", (ev) => {
      try {
        const msg = JSON.parse(ev.data as string) as WsMessage;
        batch.applyRun(msg.run);
        batch.handleEvent(msg.event);
      } catch {
        // ignore malformed frames
      }
    });

    ws.addEventListener("close", () => {
      socket = null;
      reconnectTimer = setTimeout(open, 2000);
    });

    ws.addEventListener("error", () => {
      ws.close();
    });
  };

  open();
}

export function disconnectBatchEvents(): void {
  if (reconnectTimer) clearTimeout(reconnectTimer);
  socket?.close();
  socket = null;
}
