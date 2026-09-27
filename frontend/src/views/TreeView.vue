<template>
  <main class="tree-view">
    <Breadcrumb :current="`${name}_${number}`" />

    <div class="mode-toggle" role="group" aria-label="Vergleichsmodus">
      <button
        class="mode-btn"
        :class="{ active: config.comparisonMode === 'name_size' }"
        :disabled="config.loading"
        @click="setMode('name_size')"
      >
        Name + Größe
      </button>
      <button
        class="mode-btn"
        :class="{ active: config.comparisonMode === 'name_size_hash' }"
        :disabled="config.loading"
        @click="setMode('name_size_hash')"
      >
        + Hash (erweitert)
      </button>
    </div>

    <p v-if="tree.loading" class="status">Lade Verzeichnisbaum …</p>
    <p v-else-if="tree.error" class="status error">{{ tree.error }}</p>

    <template v-else-if="tree.tree">
      <ul v-if="tree.tree.root.children.length" class="root-list">
        <TreeNode
          v-for="child in tree.tree.root.children"
          :key="child.rel_path"
          :node="child"
        />
      </ul>
      <p v-else class="status">Keine Dateien in dieser Laufwerksgruppe.</p>
    </template>
  </main>
</template>

<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useTreeStore } from "../stores/tree";
import { useConfigStore } from "../stores/config";
import type { ComparisonMode } from "../types/config";
import Breadcrumb from "../components/Breadcrumb.vue";
import TreeNode from "../components/TreeNode.vue";

const props = defineProps<{
  name: string;
  number: string;
}>();

const tree = useTreeStore();
const config = useConfigStore();

async function load() {
  if (!config.loaded) {
    await config.fetch();
  }
  await tree.fetch(props.name, props.number, config.comparisonMode);
}

onMounted(load);
watch(() => [props.name, props.number], load);

async function setMode(mode: ComparisonMode) {
  if (mode === config.comparisonMode) return;
  // Persists as the new default (per settings) and refreshes the current view.
  await config.update({ comparison_mode: mode });
  await tree.fetch(props.name, props.number, mode);
}
</script>

<style scoped>
.tree-view {
  max-width: 48rem;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
}

.status {
  color: var(--text-muted);
}

.status.error {
  color: var(--state-missing);
}

.mode-toggle {
  display: inline-flex;
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  margin-bottom: 1.25rem;
}

.mode-btn {
  border: none;
  background: var(--surface-raised);
  color: var(--text-muted);
  font-size: 0.8rem;
  padding: 0.45rem 0.85rem;
}

.mode-btn:not(:last-child) {
  border-right: 1px solid var(--border);
}

.mode-btn.active {
  background: var(--accent);
  color: #fff;
}

.root-list {
  margin: 0;
  padding: 0.5rem;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow-x: auto;
}

@media (max-width: 30rem) {
  .tree-view {
    padding: 1.25rem 0.75rem 3rem;
  }
}
</style>
