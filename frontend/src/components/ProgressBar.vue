<template>
  <div class="progress-bar" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100">
    <div class="fill" :style="{ width: `${percent}%` }" />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  done: number;
  total: number;
}>();

const percent = computed(() => {
  if (!props.total) return 0;
  return Math.min(100, Math.round((props.done / props.total) * 100));
});
</script>

<style scoped>
.progress-bar {
  height: 4px;
  border-radius: 999px;
  background: var(--surface);
  overflow: hidden;
  margin-top: 0.3rem;
}

.fill {
  height: 100%;
  background: var(--accent);
  transition: width 0.15s ease;
}
</style>
