<template>
  <Teleport to="body">
    <div class="modal-backdrop" @click.self="$emit('close')">
      <div class="modal" role="dialog" aria-modal="true">
        <header class="modal-header">
          <h2>Einstellungen</h2>
          <button class="close-btn" @click="$emit('close')">✕</button>
        </header>

        <label class="field">
          <span class="field-label">Scan-Root</span>
          <input v-model="scanRoot" type="text" placeholder="/media/user" />
          <span class="field-hint">
            Verzeichnis, in dem nach Ordnern im Muster <code>Name_Nummer{Clone}</code>
            gesucht wird (z.&nbsp;B. <code>Daten_1a</code>).
          </span>
        </label>

        <p v-if="error" class="error">{{ error }}</p>

        <footer class="modal-footer">
          <button class="btn" @click="$emit('close')">Abbrechen</button>
          <button class="btn btn-primary" :disabled="busy || !scanRoot" @click="save">
            Speichern &amp; neu scannen
          </button>
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useConfigStore } from "../stores/config";
import { useDrivesStore } from "../stores/drives";

const emit = defineEmits<{
  close: [];
}>();

const config = useConfigStore();
const drives = useDrivesStore();

const scanRoot = ref(config.scanRoot);
const busy = ref(false);
const error = ref("");

async function save() {
  busy.value = true;
  error.value = "";
  try {
    await config.update({ scan_root: scanRoot.value });
    await drives.rescan();
    emit("close");
  } catch (e) {
    error.value = e instanceof Error ? e.message : "Fehler beim Speichern.";
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 1rem;
}

.modal {
  width: min(28rem, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.25rem;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.modal-header h2 {
  font-size: 1.05rem;
  margin: 0;
}

.close-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  margin-bottom: 1rem;
}

.field-label {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text-muted);
}

.field input {
  background: var(--surface-raised);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text);
  padding: 0.5rem 0.6rem;
  font-size: 0.9rem;
  font-family: monospace;
}

.field input:focus {
  outline: none;
  border-color: var(--accent);
}

.field-hint {
  font-size: 0.75rem;
  color: var(--text-muted);
  line-height: 1.4;
}

.field-hint code {
  background: var(--surface-raised);
  border-radius: 3px;
  padding: 0.05rem 0.3rem;
}

.error {
  color: var(--state-missing);
  font-size: 0.85rem;
  margin-bottom: 0.75rem;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.6rem;
}
</style>
