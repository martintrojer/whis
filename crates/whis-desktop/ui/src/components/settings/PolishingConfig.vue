<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { settingsStore } from '../../stores/settings'
import type { Polisher } from '../../types'

const router = useRouter()

const polisher = computed(() => settingsStore.state.polisher)
const activePreset = computed(() => settingsStore.state.active_preset)
const polishPrompt = computed(() => settingsStore.state.polish_prompt)

function handlePolisherChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Polisher
  settingsStore.setPolisher(value)
}

function goToPresets() {
  router.push('/presets')
}
</script>

<template>
  <!-- Text Polishing -->
  <div class="field-row">
    <label>Text polishing</label>
    <select
      class="select-input"
      :value="polisher"
      @change="handlePolisherChange"
    >
      <option value="none">None</option>
      <option value="openai">OpenAI</option>
      <option value="mistral">Mistral</option>
      <option value="ollama">Ollama</option>
    </select>
  </div>

  <!-- Active Preset Display -->
  <div v-if="polisher !== 'none'" class="field-column">
    <div class="preset-header">
      <label>polish preset</label>
      <button class="btn-link" @click="goToPresets">
        {{ activePreset ? 'change' : 'select preset' }}
      </button>
    </div>
    <div v-if="activePreset" class="preset-display">
      <span class="preset-name">[{{ activePreset }}]</span>
      <p v-if="polishPrompt" class="preset-prompt">{{ polishPrompt }}</p>
    </div>
    <div v-else class="notice preset-notice">
      <span class="notice-marker">[i]</span>
      <p>No preset selected. Select one in <button class="btn-link inline" @click="goToPresets">presets</button> to configure polishing.</p>
    </div>
  </div>
</template>

<style scoped>
.field-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.field-row > label {
  width: 110px;
  flex-shrink: 0;
  font-size: 12px;
  color: var(--text-weak);
}

.field-row > select {
  flex: 1;
}

.field-column {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field-column > label {
  font-size: 12px;
  color: var(--text-weak);
}

.select-input {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  cursor: pointer;
  transition: border-color 0.15s ease;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23808080' d='M3 4.5L6 7.5L9 4.5'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}

.select-input:focus {
  outline: none;
  border-color: var(--accent);
}

.select-input option {
  background: var(--bg);
  color: var(--text);
}

.preset-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.preset-header label {
  font-size: 11px;
  text-transform: lowercase;
  color: var(--text-weak);
}

.preset-display {
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
}

.preset-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--accent);
}

.preset-prompt {
  font-size: 11px;
  color: var(--text-weak);
  margin-top: 8px;
  line-height: 1.4;
  white-space: pre-wrap;
  word-break: break-word;
}

.notice {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
}

.notice-marker {
  color: var(--accent);
  flex-shrink: 0;
}

.notice p {
  font-size: 12px;
  color: var(--text);
  line-height: 1.5;
  margin: 0;
}

.preset-notice {
  margin-bottom: 0;
}

.btn-link {
  background: none;
  border: none;
  color: var(--accent);
  cursor: pointer;
  font-family: var(--font);
  font-size: 11px;
  padding: 0;
}

.btn-link:hover {
  text-decoration: underline;
}

.btn-link.inline {
  display: inline;
  padding: 0;
  font-size: inherit;
}
</style>
