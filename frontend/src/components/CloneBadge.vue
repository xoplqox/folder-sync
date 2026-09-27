<template>
  <span class="clone-badge" :class="stateClass" :title="title">
    {{ clone }}
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { MatchState, RollupState } from "../types/tree";

const props = defineProps<{
  clone: string;
  state: MatchState | RollupState;
}>();

const stateClass = computed(() => `state-${props.state}`);

const labels: Record<MatchState | RollupState, string> = {
  present: "Vorhanden und synchron",
  differs: "Vorhanden, weicht ab",
  missing: "Fehlt in diesem Clone",
  identical: "Ordner ist identisch",
  partially_differs: "Enthält abweichende Dateien",
  partially_missing: "Enthält fehlende Dateien",
  empty: "Keine Dateien in diesem Zweig",
};

const title = computed(() => `Clone ${props.clone}: ${labels[props.state]}`);
</script>

<style scoped>
.clone-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 1.5rem;
  height: 1.5rem;
  padding: 0 0.35rem;
  border-radius: 5px;
  font-size: 0.7rem;
  font-weight: 600;
  text-transform: lowercase;
  letter-spacing: 0.02em;
  border: 1px solid transparent;
}

.state-present,
.state-identical {
  background: color-mix(in srgb, var(--state-present) 20%, transparent);
  color: var(--state-present);
  border-color: color-mix(in srgb, var(--state-present) 45%, transparent);
}

.state-differs,
.state-partially_differs {
  background: color-mix(in srgb, var(--state-differs) 20%, transparent);
  color: var(--state-differs);
  border-color: color-mix(in srgb, var(--state-differs) 45%, transparent);
}

.state-missing,
.state-partially_missing {
  background: color-mix(in srgb, var(--state-missing) 20%, transparent);
  color: var(--state-missing);
  border-color: color-mix(in srgb, var(--state-missing) 45%, transparent);
}

.state-empty {
  background: transparent;
  color: var(--text-muted);
  border-color: var(--border);
  border-style: dashed;
}

/* Folder rollup badges are aggregates over a whole subtree, not an exact
   per-file match — a dotted border hints at that without changing the
   color language shared with file badges. */
.state-identical,
.state-partially_differs,
.state-partially_missing {
  border-style: dotted;
}
</style>
