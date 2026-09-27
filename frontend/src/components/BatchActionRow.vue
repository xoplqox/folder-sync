<template>
  <li class="action-row" :class="`status-${action.status}`">
    <span class="action-icon">{{ action.kind.kind === "sync_file" ? "⇄" : "✕" }}</span>
    <div class="action-body">
      <div class="path">{{ action.kind.rel_path }}</div>
      <div class="detail">{{ description }}</div>
      <div v-if="action.error" class="error">{{ action.error }}</div>
    </div>
    <span class="status-badge">{{ statusLabel }}</span>
    <button
      class="remove-btn"
      :disabled="action.status === 'running'"
      title="Aus der Warteliste entfernen"
      @click="$emit('remove', action.id)"
    >
      ✕
    </button>
  </li>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { BatchAction } from "../types/batch";

const props = defineProps<{
  action: BatchAction;
}>();

defineEmits<{
  remove: [id: string];
}>();

const description = computed(() => {
  const k = props.action.kind;
  if (k.kind === "sync_file") {
    return `Sync von ${k.source_clone} → ${k.target_clones.join(", ")}`;
  }
  return `Löschen in ${k.clones.join(", ")}`;
});

const statusLabels: Record<BatchAction["status"], string> = {
  queued: "Geplant",
  running: "Läuft …",
  done: "Fertig",
  failed: "Fehlgeschlagen",
};

const statusLabel = computed(() => statusLabels[props.action.status]);
</script>

<style scoped>
.action-row {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.6rem 0.75rem;
  border-radius: 8px;
  background: var(--surface-raised);
  border: 1px solid var(--border);
}

.action-icon {
  flex-shrink: 0;
  width: 1.5rem;
  text-align: center;
  color: var(--text-muted);
}

.action-body {
  flex: 1;
  min-width: 0;
}

.path {
  font-size: 0.85rem;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail {
  font-size: 0.75rem;
  color: var(--text-muted);
}

.error {
  font-size: 0.75rem;
  color: var(--state-missing);
  margin-top: 0.15rem;
}

.status-badge {
  flex-shrink: 0;
  font-size: 0.7rem;
  color: var(--text-muted);
}

.status-failed .status-badge {
  color: var(--state-missing);
}

.status-done .status-badge {
  color: var(--state-present);
}

.remove-btn {
  flex-shrink: 0;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.8rem;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
}

.remove-btn:hover:not(:disabled) {
  background: var(--surface);
  color: var(--state-missing);
}
</style>
