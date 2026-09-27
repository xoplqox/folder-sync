<template>
  <Teleport to="body">
    <div class="modal-backdrop" @click.self="$emit('close')">
      <div class="modal" role="dialog" aria-modal="true">
        <header class="modal-header">
          <h2>Konflikt lösen</h2>
          <button class="close-btn" @click="$emit('close')">✕</button>
        </header>

        <p class="path">{{ node.rel_path }}</p>
        <p class="hint">
          Diese Datei unterscheidet sich zwischen den Clonen. Wähle die Version, die als
          Quelle für alle anderen Clone übernommen werden soll.
        </p>

        <ul class="clone-list">
          <li v-for="c in candidates" :key="c.clone" class="clone-option">
            <label>
              <input v-model="chosen" type="radio" name="chosen-clone" :value="c.clone" />
              <span class="clone-letter">{{ c.clone }}</span>
              <CloneBadge :clone="c.clone" :state="c.state" />
              <span class="meta">
                <span v-if="c.size !== null">{{ formatBytes(c.size) }}</span>
                <span v-if="c.mtime_unix !== null">· {{ formatDate(c.mtime_unix) }}</span>
              </span>
            </label>
          </li>
        </ul>

        <p v-if="error" class="error">{{ error }}</p>

        <footer class="modal-footer">
          <button class="btn" @click="$emit('close')">Abbrechen</button>
          <button class="btn btn-primary" :disabled="!chosen || busy" @click="confirm">
            Als Quelle übernehmen
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import { useBatchStore } from "../stores/batch";
import CloneBadge from "./CloneBadge.vue";
import type { MergedNode } from "../types/tree";

const props = defineProps<{
  node: MergedNode;
}>();

const emit = defineEmits<{
  close: [];
  resolved: [];
}>();

const route = useRoute();
const batch = useBatchStore();

const candidates = computed(() => props.node.clones.filter((c) => c.state !== "missing"));

const chosen = ref<string | null>(candidates.value[0]?.clone ?? null);
const busy = ref(false);
const error = ref("");

async function confirm() {
  if (!chosen.value) return;
  busy.value = true;
  error.value = "";
  try {
    await batch.queue({
      kind: "resolve_conflict",
      group_name: route.params.name as string,
      group_number: route.params.number as string,
      rel_path: props.node.rel_path,
      chosen_clone: chosen.value,
    });
    emit("resolved");
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Fehler beim Einreihen.";
  } finally {
    busy.value = false;
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB"];
  let value = bytes / 1024;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${value.toFixed(1)} ${units[unit]}`;
}

function formatDate(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toLocaleString("de-DE");
}
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.modal {
  width: min(28rem, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.25rem;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.modal-header h2 {
  font-size: 1.05rem;
  margin: 0;
}

.close-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 1rem;
}

.path {
  font-family: monospace;
  font-size: 0.85rem;
  color: var(--text-muted);
  word-break: break-all;
}

.hint {
  font-size: 0.85rem;
  color: var(--text-muted);
  line-height: 1.5;
  margin-bottom: 1rem;
}

.clone-list {
  list-style: none;
  margin: 0 0 1rem;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.clone-option label {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.5rem 0.6rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
}

.clone-option label:hover {
  border-color: var(--accent);
}

.clone-letter {
  font-weight: 600;
  text-transform: uppercase;
  width: 1rem;
}

.meta {
  margin-left: auto;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.error {
  color: var(--state-missing);
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
}
</style>
