<template>
  <main class="hero">
    <h1>folder-sync</h1>

    <p v-if="drives.loading && !drives.loaded" class="status">Suche Laufwerke …</p>
    <p v-else-if="drives.error" class="status error">{{ drives.error }}</p>

    <template v-else-if="drives.loaded">
      <EmptyState v-if="drives.groups.length === 0" :scan-root="drives.scanRoot" />

      <div v-else class="drive-list">
        <DriveGroupCard
          v-for="group in sortedGroups"
          :key="`${group.name}_${group.number}`"
          :group="group"
        />
      </div>
    </template>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted } from "vue";
import { useDrivesStore } from "../stores/drives";
import DriveGroupCard from "../components/DriveGroupCard.vue";
import EmptyState from "../components/EmptyState.vue";

const drives = useDrivesStore();

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

h1 {
  text-align: center;
  margin-bottom: 2rem;
  font-size: 1.5rem;
  letter-spacing: -0.01em;
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
