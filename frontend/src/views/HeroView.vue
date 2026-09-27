<template>
  <main class="hero">
    <div class="hero-header">
      <h1>folder-sync</h1>
      <button class="settings-btn" title="Einstellungen" @click="settingsOpen = true">⚙</button>
    </div>

    <p v-if="drives.loading && !drives.loaded" class="status">Suche Laufwerke …</p>
    <p v-else-if="drives.error" class="status error">{{ drives.error }}</p>

    <template v-else-if="drives.loaded">
      <EmptyState v-if="drives.groups.length === 0" :scan-root="drives.scanRoot">
        <template #actions>
          <button class="btn btn-primary" @click="settingsOpen = true">Scan-Root ändern</button>
          <button class="btn" @click="drives.rescan()">Erneut scannen</button>
        </template>
      </EmptyState>

      <div v-else class="drive-list">
        <DriveGroupCard
          v-for="group in sortedGroups"
          :key="`${group.name}_${group.number}`"
          :group="group"
        />
      </div>
    </template>

    <SettingsDialog v-if="settingsOpen" @close="settingsOpen = false" />
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useDrivesStore } from "../stores/drives";
import DriveGroupCard from "../components/DriveGroupCard.vue";
import EmptyState from "../components/EmptyState.vue";
import SettingsDialog from "../components/SettingsDialog.vue";

const drives = useDrivesStore();
const settingsOpen = ref(false);

onMounted(() => {
  drives.fetch();
});

const sortedGroups = computed(() =>
  [...drives.groups].sort((a, b) =>
    `${a.name}_${a.number}`.localeCompare(`${b.name}_${b.number}`, "de", {
      numeric: true,
      sensitivity: "base",
    }),
  ),
);
</script>

<style scoped>
.hero {
  max-width: 40rem;
  margin: 0 auto;
  padding: 3rem 1.5rem;
}

.hero-header {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 2rem;
}

h1 {
  text-align: center;
  font-size: 1.5rem;
  letter-spacing: -0.01em;
  margin: 0;
}

.settings-btn {
  position: absolute;
  right: 0;
  border: 1px solid var(--border);
  background: var(--surface-raised);
  color: var(--text-muted);
  border-radius: 6px;
  width: 2rem;
  height: 2rem;
  font-size: 0.95rem;
}

.settings-btn:hover {
  border-color: var(--accent);
  color: var(--text);
}

.status {
  text-align: center;
  color: var(--text-muted);
}

.status.error {
  color: var(--state-missing);
}

.drive-list {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
</style>
