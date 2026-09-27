<template>
  <Teleport to="body">
    <div v-if="open" class="backdrop" @click="$emit('close')" />
    <aside class="drawer" :class="{ open }">
      <header class="drawer-header">
        <h2>Batch-Warteliste ({{ batch.count }})</h2>
        <button class="close-btn" @click="$emit('close')">✕</button>
      </header>

      <p v-if="batch.actions.length === 0" class="empty">
        Keine geplanten Aktionen. Klicke im Verzeichnisbaum auf Sync- oder
        Löschen-Aktionen, um sie hier einzureihen.
      </p>

      <div v-else class="groups">
        <section v-for="group in groups" :key="group.id ?? 'ungrouped'" class="group">
          <div v-if="group.id" class="group-header">
            <span>Ordner-Aktion ({{ group.actions.length }} Dateien)</span>
            <button class="remove-group-btn" @click="removeGroup(group.id)">
              Alle entfernen
            </button>
          </div>
          <ul class="action-list">
            <BatchActionRow
              v-for="action in group.actions"
              :key="action.id"
              :action="action"
              @remove="remove"
            />
          </ul>
        </section>
      </div>

      <footer v-if="batch.actions.length > 0" class="drawer-footer">
        <p v-if="batch.lastCompleted" class="summary">
          Letzter Lauf: {{ batch.lastCompleted.succeeded }} erfolgreich, {{ batch.lastCompleted.failed }} fehlgeschlagen.
        </p>
        <p v-if="config.readOnly" class="readonly-note">
          Read-Only-Modus: Ausführen ist deaktiviert.
        </p>
        <div class="footer-actions">
          <button
            v-if="!batch.isRunning"
            class="btn btn-primary"
            :disabled="batch.queuedCount === 0 || config.readOnly"
            @click="start"
          >
            Batch starten ({{ batch.queuedCount }})
          </button>
          <button v-else class="btn" @click="cancel">Abbrechen</button>
        </div>
      </footer>
    </aside>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useBatchStore } from "../stores/batch";
import { useConfigStore } from "../stores/config";
import BatchActionRow from "./BatchActionRow.vue";
import type { BatchAction } from "../types/batch";

defineProps<{
  open: boolean;
}>();

defineEmits<{
  close: [];
}>();

const batch = useBatchStore();
const config = useConfigStore();

interface Group {
  id: string | null;
  actions: BatchAction[];
}

const groups = computed<Group[]>(() => {
  const result: Group[] = [];
  const byId = new Map<string, Group>();
  for (const action of batch.actions) {
    if (!action.group_id) {
      result.push({ id: null, actions: [action] });
      continue;
    }
    let group = byId.get(action.group_id);
    if (!group) {
      group = { id: action.group_id, actions: [] };
      byId.set(action.group_id, group);
      result.push(group);
    }
    group.actions.push(action);
  }
  return result;
});

function remove(id: string) {
  batch.remove(id);
}

function removeGroup(id: string) {
  batch.removeGroup(id);
}

function start() {
  batch.start();
}

function cancel() {
  batch.cancel();
}
</script>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  z-index: 40;
}

.drawer {
  position: fixed;
  top: 0;
  right: 0;
  bottom: 0;
  width: min(24rem, 100vw);
  background: var(--surface);
  border-left: 1px solid var(--border);
  z-index: 50;
  display: flex;
  flex-direction: column;
  transform: translateX(100%);
  transition: transform 0.2s ease;
}

.drawer.open {
  transform: translateX(0);
}

.drawer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.25rem;
  border-bottom: 1px solid var(--border);
}

.drawer-header h2 {
  font-size: 1rem;
  margin: 0;
}

.close-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 1rem;
  padding: 0.25rem 0.5rem;
}

.close-btn:hover {
  color: var(--text);
}

.empty {
  padding: 1.5rem 1.25rem;
  color: var(--text-muted);
  font-size: 0.85rem;
  line-height: 1.5;
}

.groups {
  overflow-y: auto;
  padding: 1rem 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-bottom: 0.4rem;
}

.remove-group-btn {
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 0.75rem;
  padding: 0.1rem 0.3rem;
}

.action-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.drawer-footer {
  border-top: 1px solid var(--border);
  padding: 1rem 1.25rem;
}

.summary {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin: 0 0 0.5rem;
}

.readonly-note {
  font-size: 0.8rem;
  color: var(--state-differs);
  margin: 0 0 0.5rem;
}

.footer-actions {
  display: flex;
}

.footer-actions .btn {
  width: 100%;
}
</style>
