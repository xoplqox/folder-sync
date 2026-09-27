<template>
  <li class="node" :class="node.kind">
    <div class="row" @click="node.kind === 'folder' && toggle()">
      <span
        v-if="node.kind === 'folder'"
        class="chevron"
        :class="{ open: expanded }"
        >▸</span
      >
      <span v-else class="chevron-spacer" />

      <svg
        v-if="node.kind === 'folder'"
        class="icon folder-icon"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M3 6.5A1.5 1.5 0 0 1 4.5 5h4.4l1.6 2H19.5A1.5 1.5 0 0 1 21 8.5v9A1.5 1.5 0 0 1 19.5 19h-15A1.5 1.5 0 0 1 3 17.5v-11Z"
          fill="currentColor"
        />
      </svg>
      <svg
        v-else
        class="icon file-icon"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M6 3h8l4 4v14a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1Z"
          stroke="currentColor"
          stroke-width="1.4"
        />
      </svg>

      <span class="name">{{ node.name }}</span>

      <span class="badges">
        <CloneBadge
          v-for="b in badges"
          :key="b.clone"
          :clone="b.clone"
          :state="b.state"
        />
      </span>

      <span class="node-actions" @click.stop>
        <span v-if="hasConflict" class="conflict-pill" title="Konflikt: unterschiedliche Versionen vorhanden">
          ⚠ Konflikt
        </span>
        <button
          v-if="canSync"
          class="action-btn"
          title="In alle Clone synchronisieren"
          :disabled="busy"
          @click="doSync"
        >
          ⇄
        </button>
        <button
          v-if="canDelete"
          class="action-btn danger"
          title="In allen Clonen löschen"
          :disabled="busy"
          @click="doDelete"
        >
          🗑
        </button>
      </span>
    </div>

    <p v-if="feedback" class="feedback">{{ feedback }}</p>

    <ul v-if="node.kind === 'folder' && expanded" class="children">
      <TreeNode v-for="child in node.children" :key="child.rel_path" :node="child" />
    </ul>
  </li>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute } from "vue-router";
import type { MergedNode } from "../types/tree";
import CloneBadge from "./CloneBadge.vue";
import { useBatchStore } from "../stores/batch";

const props = defineProps<{
  node: MergedNode;
}>();

const expanded = ref(false);
const busy = ref(false);
const feedback = ref("");

function toggle() {
  expanded.value = !expanded.value;
}

const badges = computed(() => {
  if (props.node.kind === "file") {
    return props.node.clones.map((c) => ({ clone: c.clone, state: c.state }));
  }
  return (props.node.rollup ?? []).map((r) => ({
    clone: r.clone,
    state: r.state,
  }));
});

const route = useRoute();
const batch = useBatchStore();

const hasConflict = computed(
  () => props.node.kind === "file" && props.node.clones.some((c) => c.state === "differs"),
);

const canSync = computed(() => {
  if (props.node.kind === "folder") return true;
  if (hasConflict.value) return false;
  return props.node.clones.some((c) => c.state === "missing") && props.node.clones.some((c) => c.state === "present");
});

const canDelete = computed(() => {
  if (props.node.kind === "folder") return true;
  return props.node.clones.some((c) => c.state !== "missing");
});

function showFeedback(text: string) {
  feedback.value = text;
  setTimeout(() => {
    if (feedback.value === text) feedback.value = "";
  }, 4000);
}

async function doSync() {
  busy.value = true;
  try {
    const kind = props.node.kind === "folder" ? "sync_folder" : "sync_file";
    const skipped = await batch.queue({
      kind,
      group_name: route.params.name as string,
      group_number: route.params.number as string,
      rel_path: props.node.rel_path,
    });
    if (props.node.kind === "folder") {
      showFeedback(skipped > 0 ? `Eingereiht. ${skipped} Datei(en) benötigen die Konfliktlösung.` : "Eingereiht.");
    }
  } catch (e) {
    showFeedback(e instanceof Error ? e.message : "Fehler beim Einreihen.");
  } finally {
    busy.value = false;
  }
}

async function doDelete() {
  busy.value = true;
  try {
    const kind = props.node.kind === "folder" ? "delete_folder" : "delete_file";
    await batch.queue({
      kind,
      group_name: route.params.name as string,
      group_number: route.params.number as string,
      rel_path: props.node.rel_path,
    });
    if (props.node.kind === "folder") {
      showFeedback("Eingereiht.");
    }
  } catch (e) {
    showFeedback(e instanceof Error ? e.message : "Fehler beim Einreihen.");
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.node {
  list-style: none;
}

.row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  cursor: default;
}

.node.folder > .row {
  cursor: pointer;
}

.node.folder > .row:hover {
  background: var(--surface-raised);
}

.chevron {
  width: 0.9rem;
  flex-shrink: 0;
  color: var(--text-muted);
  transition: transform 0.12s ease;
  font-size: 0.7rem;
}

.chevron.open {
  transform: rotate(90deg);
}

.chevron-spacer {
  width: 0.9rem;
  flex-shrink: 0;
}

.icon {
  width: 1.05rem;
  height: 1.05rem;
  flex-shrink: 0;
}

.folder-icon {
  color: var(--accent);
}

.file-icon {
  color: var(--text-muted);
}

.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 0.9rem;
}

.badges {
  display: flex;
  gap: 0.25rem;
  flex-shrink: 0;
}

.node-actions {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  flex-shrink: 0;
  margin-left: 0.5rem;
}

.conflict-pill {
  font-size: 0.7rem;
  color: var(--state-differs);
  white-space: nowrap;
}

.action-btn {
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text-muted);
  border-radius: 5px;
  width: 1.6rem;
  height: 1.6rem;
  font-size: 0.8rem;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.action-btn:hover:not(:disabled) {
  border-color: var(--accent);
  color: var(--text);
}

.action-btn.danger:hover:not(:disabled) {
  border-color: var(--state-missing);
  color: var(--state-missing);
}

.action-btn:disabled {
  opacity: 0.4;
}

.feedback {
  margin: 0 0 0.25rem 2.4rem;
  font-size: 0.75rem;
  color: var(--text-muted);
}

.children {
  margin: 0;
  padding-left: 1.4rem;
  border-left: 1px solid var(--border);
  margin-left: 0.65rem;
}
</style>
