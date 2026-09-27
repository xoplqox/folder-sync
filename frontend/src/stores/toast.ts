import { defineStore } from "pinia";

export type ToastKind = "info" | "success" | "error";

export interface Toast {
  id: number;
  message: string;
  kind: ToastKind;
}

let nextId = 1;

export const useToastStore = defineStore("toast", {
  state: () => ({
    items: [] as Toast[],
  }),
  actions: {
    push(message: string, kind: ToastKind = "info", durationMs = 5000) {
      const id = nextId++;
      this.items.push({ id, message, kind });
      setTimeout(() => this.dismiss(id), durationMs);
    },
    dismiss(id: number) {
      this.items = this.items.filter((t) => t.id !== id);
    },
  },
});
