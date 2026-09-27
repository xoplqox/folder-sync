<template>
  <nav class="breadcrumb">
    <RouterLink to="/">Laufwerke</RouterLink>
    <span class="sep">›</span>
    <span class="current">{{ current }}</span>
  </nav>
</template>

<script setup lang="ts">
defineProps<{
  current: string;
}>();
</script>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.9rem;
  margin-bottom: 1.5rem;
}

.breadcrumb a {
  color: var(--text-muted);
  text-decoration: none;
}

.breadcrumb a:hover {
  color: var(--accent);
}

.sep {
  color: var(--text-muted);
}

.current {
  color: var(--text);
  font-weight: 600;
}
</style>
