<template>
  <div class="app-shell">
    <div v-if="config.loaded && config.readOnly" class="readonly-banner">
      🔒 Read-Only-Modus aktiv — Untersuchen und Einreihen ist möglich, Sync/Löschen ausführen ist deaktiviert.
    </div>

    <router-view />

    <button class="batch-toggle" @click="drawerOpen = !drawerOpen">
      Batch
      <span v-if="batch.count > 0" class="badge">{{ batch.count }}</span>
    </button>

    <BatchQueueDrawer :open="drawerOpen" @close="drawerOpen = false" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useBatchStore } from "./stores/batch";
import { useConfigStore } from "./stores/config";
import { connectBatchEvents } from "./api/ws";
import BatchQueueDrawer from "./components/BatchQueueDrawer.vue";

const batch = useBatchStore();
const config = useConfigStore();
const drawerOpen = ref(false);

onMounted(() => {
  batch.fetch();
  config.fetch();
  connectBatchEvents();
});
</script>

<style>
.app-shell {
  min-height: 100vh;
}

.readonly-banner {
  position: sticky;
  top: 0;
  z-index: 20;
  padding: 0.6rem 1rem;
  text-align: center;
  font-size: 0.85rem;
  font-weight: 600;
  background: color-mix(in srgb, var(--state-differs) 20%, var(--surface));
  color: var(--state-differs);
  border-bottom: 1px solid var(--state-differs);
}

.batch-toggle {
  position: fixed;
  right: 1.5rem;
  bottom: 1.5rem;
  z-index: 30;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.65rem 1.1rem;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-raised);
  color: var(--text);
  font-size: 0.85rem;
  font-weight: 600;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
}

.batch-toggle:hover {
  border-color: var(--accent);
}

.batch-toggle .badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.3rem;
  height: 1.3rem;
  padding: 0 0.35rem;
  border-radius: 999px;
  background: var(--accent);
  color: #fff;
  font-size: 0.7rem;
}
</style>
