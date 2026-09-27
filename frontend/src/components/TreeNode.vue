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
    </div>

    <ul v-if="node.kind === 'folder' && expanded" class="children">
      <TreeNode v-for="child in node.children" :key="child.rel_path" :node="child" />
    </ul>
  </li>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import type { MergedNode } from "../types/tree";
import CloneBadge from "./CloneBadge.vue";

const props = defineProps<{
  node: MergedNode;
}>();

const expanded = ref(false);

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

.children {
  margin: 0;
  padding-left: 1.4rem;
  border-left: 1px solid var(--border);
  margin-left: 0.65rem;
}
</style>
