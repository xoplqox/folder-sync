<template>
  <div class="empty-state">
    <svg
      class="icon"
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
        stroke-width="1.4"
      />
      <path d="M6 6l4-3h4l4 3" stroke="currentColor" stroke-width="1.4" />
      <circle cx="7" cy="12" r="1.4" fill="currentColor" />
    </svg>
    <h2>Keine Laufwerke gefunden</h2>
    <p>
      Bitte prüfe, ob deine Backup-Festplatten angeschlossen und gemountet
      sind. Erwartetes Namensmuster:
      <code>Name_Nummer{Clone}</code> (z.&nbsp;B. <code>Daten_1a</code>).
    </p>
    <p class="scan-root">
      Aktuell durchsuchter Pfad: <code>{{ scanRoot }}</code>
    </p>
    <div class="actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  scanRoot: string;
}>();
</script>

<style scoped>
.empty-state {
  max-width: 32rem;
  margin: 3rem auto 0;
  padding: 2rem;
  text-align: center;
  color: var(--text-muted);
}

.icon {
  width: 3rem;
  height: 3rem;
  color: var(--text-muted);
  margin-bottom: 1rem;
}

h2 {
  color: var(--text);
  margin: 0 0 0.75rem;
}

p {
  line-height: 1.5;
}

code {
  background: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0.05rem 0.35rem;
  font-size: 0.9em;
}

.scan-root {
  font-size: 0.9rem;
}

.actions {
  margin-top: 1.5rem;
  display: flex;
  justify-content: center;
  gap: 0.75rem;
}
</style>
