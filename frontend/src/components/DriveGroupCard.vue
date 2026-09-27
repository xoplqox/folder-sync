<template>
  <RouterLink
    class="drive-card"
    :to="{ name: 'tree', params: { name: group.name, number: group.number } }"
  >
    <svg
      class="drive-icon"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect
        x="2"
        y="6"
        width="20"
        height="12"
        rx="2.5"
        stroke="currentColor"
        stroke-width="1.6"
      />
      <circle cx="7" cy="12" r="1.6" fill="currentColor" />
      <rect x="11.5" y="11" width="8" height="2" rx="1" fill="currentColor" />
    </svg>
    <span class="label">{{ group.name }}_{{ group.number }}</span>
    <span class="clone-count" :title="`${group.clone_count} Clone(s)`">
      {{ group.clone_count }}
    </span>
  </RouterLink>
</template>

<script setup lang="ts">
import type { DriveGroupSummary } from "../types/drive";

defineProps<{
  group: DriveGroupSummary;
}>();
</script>

<style scoped>
.drive-card {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  padding: 1rem 1.25rem;
  background: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: 10px;
  color: var(--text);
  text-decoration: none;
  transition:
    border-color 0.15s ease,
    transform 0.1s ease;
}

.drive-card:hover {
  border-color: var(--accent);
  transform: translateY(-1px);
}

.drive-icon {
  width: 1.5rem;
  height: 1.5rem;
  flex-shrink: 0;
  color: var(--accent);
}

.label {
  flex: 1;
  font-size: 1.05rem;
  font-weight: 600;
}

.clone-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.75rem;
  height: 1.75rem;
  padding: 0 0.5rem;
  border-radius: 999px;
  background: var(--surface);
  border: 1px solid var(--border);
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--text-muted);
}
</style>
