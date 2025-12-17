<script setup lang="ts">
import { ref, computed, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { settingsStore } from '../../stores/settings'

const ollamaUrl = computed(() => settingsStore.state.ollama_url)
const ollamaModel = computed(() => settingsStore.state.ollama_model)

// Connection state
const ollamaStatus = ref<'unknown' | 'connecting' | 'connected' | 'not-installed' | 'error'>('unknown')
const ollamaStatusMessage = ref('')
const ollamaModels = ref<string[]>([])

// Pull state
const pullModelName = ref('phi3')
const pullingModel = ref(false)
const pullStatus = ref('')
const pullProgress = ref<{ downloaded: number; total: number } | null>(null)
let unlistenPull: UnlistenFn | null = null

// Cleanup on unmount
onUnmounted(() => {
  if (unlistenPull) {
    unlistenPull()
    unlistenPull = null
  }
})

// Format pull progress
const pullProgressPercent = computed(() => {
  if (!pullProgress.value || pullProgress.value.total === 0) return 0
  return Math.round((pullProgress.value.downloaded / pullProgress.value.total) * 100)
})

const pullProgressText = computed(() => {
  if (!pullProgress.value) return ''
  const { downloaded, total } = pullProgress.value
  const downloadedMB = (downloaded / 1_000_000).toFixed(0)
  const totalMB = (total / 1_000_000).toFixed(0)
  return `${downloadedMB} MB / ${totalMB} MB`
})

async function testOllamaConnection() {
  const url = ollamaUrl.value || ''
  ollamaStatus.value = 'connecting'
  ollamaStatusMessage.value = 'Connecting...'

  try {
    // First check if already running
    const isRunning = await invoke<boolean>('test_ollama_connection', { url })
    if (isRunning) {
      ollamaStatus.value = 'connected'
      ollamaStatusMessage.value = 'Connected'
      await loadOllamaModels()
      return
    }

    // Try to auto-start
    ollamaStatusMessage.value = 'Starting Ollama...'
    const result = await invoke<string>('start_ollama', { url })
    if (result === 'started' || result === 'running') {
      ollamaStatus.value = 'connected'
      ollamaStatusMessage.value = result === 'started' ? 'Started & connected' : 'Connected'
      await loadOllamaModels()
    }
  } catch (e) {
    const error = String(e)
    if (error.toLowerCase().includes('not installed')) {
      ollamaStatus.value = 'not-installed'
      ollamaStatusMessage.value = 'Ollama not installed'
    } else {
      ollamaStatus.value = 'error'
      ollamaStatusMessage.value = error
    }
  }
}

async function loadOllamaModels() {
  const url = ollamaUrl.value || ''
  try {
    ollamaModels.value = await invoke<string[]>('list_ollama_models', { url })
    // If no model selected but models exist, select first one
    const firstModel = ollamaModels.value[0]
    if (!ollamaModel.value && firstModel) {
      settingsStore.setOllamaModel(firstModel)
    }
  } catch (e) {
    console.error('Failed to load Ollama models:', e)
    ollamaModels.value = []
  }
}

async function pullOllamaModel() {
  if (!pullModelName.value.trim()) {
    pullStatus.value = 'Enter a model name'
    return
  }

  const url = ollamaUrl.value || ''
  pullingModel.value = true
  pullProgress.value = null
  pullStatus.value = ''

  // Listen for progress events
  unlistenPull = await listen<{ downloaded: number; total: number }>('ollama-pull-progress', (event) => {
    pullProgress.value = event.payload
  })

  try {
    await invoke('pull_ollama_model', { url, model: pullModelName.value.trim() })
    await loadOllamaModels()
    // Auto-select the pulled model
    settingsStore.setOllamaModel(pullModelName.value.trim())
    pullStatus.value = 'Model ready!'
    setTimeout(() => pullStatus.value = '', 3000)
  } catch (e) {
    pullStatus.value = String(e)
  } finally {
    pullingModel.value = false
    pullProgress.value = null
    if (unlistenPull) {
      unlistenPull()
      unlistenPull = null
    }
  }
}

function handleOllamaModelChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  settingsStore.setOllamaModel(value || null)
}
</script>

<template>
  <!-- URL and Connection Test -->
  <div class="field-row">
    <label>Ollama URL</label>
    <div class="url-config">
      <input
        type="text"
        :value="ollamaUrl || ''"
        @input="settingsStore.setOllamaUrl(($event.target as HTMLInputElement).value || null)"
        placeholder="http://localhost:11434"
        spellcheck="false"
        aria-label="Ollama server URL"
      />
      <button
        class="ollama-ping-btn"
        @click="testOllamaConnection"
        :disabled="ollamaStatus === 'connecting'"
      >
        {{ ollamaStatus === 'connecting' ? '...' : 'ping' }}
      </button>
    </div>
  </div>
  <p v-if="ollamaStatusMessage" class="hint ollama-hint" :class="{ success: ollamaStatus === 'connected', error: ollamaStatus === 'error' || ollamaStatus === 'not-installed' }" role="status" aria-live="polite">
    {{ ollamaStatusMessage }}
    <a v-if="ollamaStatus === 'not-installed'" href="https://ollama.ai" target="_blank"> → install</a>
  </p>

  <!-- Model Selection (only if connected) -->
  <div v-if="ollamaStatus === 'connected'" class="field-row">
    <label>Model</label>
    <select
      class="select-input"
      :value="ollamaModel || ''"
      @change="handleOllamaModelChange"
      aria-label="Select Ollama model"
    >
      <option value="" disabled>Select a model...</option>
      <option v-for="model in ollamaModels" :key="model" :value="model">
        {{ model }}
      </option>
    </select>
  </div>
  <p v-if="ollamaStatus === 'connected' && ollamaModels.length > 0" class="hint ollama-hint">
    First polish may be slow while model loads into memory.
  </p>

  <!-- No models warning -->
  <div v-if="ollamaStatus === 'connected' && ollamaModels.length === 0" class="notice ollama-notice">
    <span class="notice-marker">[!]</span>
    <p>No models installed. Pull a model below to get started.</p>
  </div>

  <!-- Pull Model UI (only if connected) -->
  <div v-if="ollamaStatus === 'connected'" class="field-row">
    <label>Pull model</label>
    <div class="pull-model-input">
      <input
        type="text"
        v-model="pullModelName"
        placeholder="phi3"
        spellcheck="false"
        :disabled="pullingModel"
        aria-label="Model name to pull"
      />
      <button
        class="btn-primary"
        @click="pullOllamaModel"
        :disabled="pullingModel || !pullModelName.trim()"
      >
        {{ pullingModel ? `${pullProgressPercent}%` : 'Pull' }}
      </button>
    </div>
  </div>
  <p v-if="pullProgress" class="hint ollama-hint">
    {{ pullProgressText }}
  </p>
  <p v-else-if="pullStatus" class="hint ollama-hint" :class="{ success: pullStatus.includes('ready'), error: !pullStatus.includes('ready') }">
    {{ pullStatus }}
  </p>
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

.field-row > select,
.field-row > input {
  flex: 1;
}

.url-config {
  display: flex;
  gap: 8px;
  flex: 1;
}

.url-config input {
  flex: 1;
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  transition: border-color 0.15s ease;
}

.url-config input:focus {
  outline: none;
  border-color: var(--accent);
}

.url-config input::placeholder {
  color: var(--text-weak);
}

.ollama-ping-btn {
  min-width: 56px;
  padding: 10px 16px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text-weak);
  cursor: pointer;
  transition: all 0.15s ease;
}

.ollama-ping-btn:hover:not(:disabled) {
  border-color: var(--text-weak);
  color: var(--text);
}

.ollama-ping-btn:disabled {
  cursor: wait;
  color: var(--accent);
  border-color: var(--accent);
}

.hint {
  font-size: 11px;
  color: var(--text-weak);
  margin: 0;
}

.hint.success {
  color: #4ade80;
}

.hint.error {
  color: #f87171;
}

.ollama-hint {
  margin-left: 122px;
}

.ollama-hint a {
  color: var(--accent);
  text-decoration: none;
}

.ollama-hint a:hover {
  text-decoration: underline;
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

.ollama-notice {
  margin-left: 122px;
  margin-bottom: 0;
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

.pull-model-input {
  display: flex;
  gap: 8px;
  flex: 1;
}

.pull-model-input input {
  flex: 1;
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  transition: border-color 0.15s ease;
}

.pull-model-input input:focus {
  outline: none;
  border-color: var(--accent);
}

.pull-model-input input::placeholder {
  color: var(--text-weak);
}

.pull-model-input input:disabled {
  opacity: 0.6;
}

.btn-primary {
  padding: 10px 20px;
  background: var(--accent);
  border: none;
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--bg);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-primary:hover:not(:disabled) {
  filter: brightness(1.1);
}

.btn-primary:focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
