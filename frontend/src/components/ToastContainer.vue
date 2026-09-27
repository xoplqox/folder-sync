<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div v-for="t in toast.items" :key="t.id" class="toast" :class="`kind-${t.kind}`">
          <span>{{ t.message }}</span>
          <button class="dismiss" @click="toast.dismiss(t.id)">✕</button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToastStore } from "../stores/toast";

const toast = useToastStore();
</script>

<style scoped>
.toast-container {
  position: fixed;
  left: 1.5rem;
  bottom: 1.5rem;
  z-index: 70;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  max-width: min(22rem, calc(100vw - 3rem));
}

.toast {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.65rem 0.85rem;
  border-radius: 8px;
  background: var(--surface-raised);
  border: 1px solid var(--border);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
  font-size: 0.85rem;
}

.toast span {
  flex: 1;
}

.kind-success {
  border-color: color-mix(in srgb, var(--state-present) 45%, transparent);
}

.kind-error {
  border-color: color-mix(in srgb, var(--state-missing) 45%, transparent);
}

.dismiss {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.75rem;
  flex-shrink: 0;
}

.toast-enter-active,
.toast-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(0.5rem);
}
</style>
