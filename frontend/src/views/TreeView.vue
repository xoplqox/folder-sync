<template>
  <main class="tree-view">
    <Breadcrumb :current="`${name}_${number}`" />

    <p v-if="tree.loading" class="status">Lade Verzeichnisbaum …</p>
    <p v-else-if="tree.error" class="status error">{{ tree.error }}</p>

    <template v-else-if="tree.tree">
      <div class="legend">
        <span
          >Modus:
          {{
            tree.tree.comparison_mode === "name_size_hash"
              ? "Name + Größe + Hash"
              : "Name + Größe"
          }}</span
        >
      </div>

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
import Breadcrumb from "../components/Breadcrumb.vue";
import TreeNode from "../components/TreeNode.vue";

const props = defineProps<{
  name: string;
  number: string;
}>();

const tree = useTreeStore();

function load() {
  tree.fetch(props.name, props.number);
}

onMounted(load);
watch(() => [props.name, props.number], load);
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

.legend {
  font-size: 0.8rem;
  color: var(--text-muted);
  margin-bottom: 1rem;
}

.root-list {
  margin: 0;
  padding: 0;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 0.5rem;
}
</style>
