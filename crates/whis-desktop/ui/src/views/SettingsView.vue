<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { settingsStore } from '../stores/settings'
import type { Provider, Polisher } from '../types'
import {
  ModeCards,
  CloudProviderConfig,
  LocalWhisperConfig,
  RemoteWhisperConfig,
  OllamaConfig,
  PolishingConfig,
  type TranscriptionMode,
} from '../components'

const savingStatus = ref('')
const showAdvanced = ref(false)

// Settings from store
const provider = computed(() => settingsStore.state.provider)
const language = computed(() => settingsStore.state.language)
const apiKeys = computed(() => settingsStore.state.api_keys)
const remoteWhisperUrl = computed(() => settingsStore.state.remote_whisper_url)
const polisher = computed(() => settingsStore.state.polisher)

// Transcription mode: cloud vs local
const transcriptionMode = ref<TranscriptionMode>(
  ['local-whisper', 'remote-whisper'].includes(provider.value) ? 'local' : 'cloud'
)

// Whisper model validation (for local provider)
const whisperModelValid = ref(false)

// Watch for provider changes to validate model
watch(provider, async () => {
  if (provider.value === 'local-whisper') {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      whisperModelValid.value = await invoke<boolean>('is_whisper_model_valid')
    } catch {
      whisperModelValid.value = false
    }
  }
}, { immediate: true })

// Configuration status - shows if ready or what's needed
const configStatus = computed(() => {
  if (transcriptionMode.value === 'cloud') {
    const key = apiKeys.value[provider.value] || ''
    if (!key) return { ready: false, message: 'API key required' }
    return { ready: true, message: 'Ready' }
  } else {
    if (provider.value === 'local-whisper' && !whisperModelValid.value) {
      return { ready: false, message: 'Model required' }
    }
    if (provider.value === 'remote-whisper' && !remoteWhisperUrl.value) {
      return { ready: false, message: 'Server URL required' }
    }
    return { ready: true, message: provider.value === 'local-whisper' ? 'Local Whisper' : 'Remote Whisper' }
  }
})

// Local mode options for dropdown
const localModeOptions = [
  { value: 'local-whisper', label: 'Local Whisper' },
  { value: 'remote-whisper', label: 'Remote Whisper' },
]

// Common language codes for the dropdown
const languageOptions = [
  { value: null, label: 'Auto-detect' },
  { value: 'en', label: 'English (en)' },
  { value: 'de', label: 'German (de)' },
  { value: 'fr', label: 'French (fr)' },
  { value: 'es', label: 'Spanish (es)' },
  { value: 'it', label: 'Italian (it)' },
  { value: 'pt', label: 'Portuguese (pt)' },
  { value: 'nl', label: 'Dutch (nl)' },
  { value: 'pl', label: 'Polish (pl)' },
  { value: 'ru', label: 'Russian (ru)' },
  { value: 'ja', label: 'Japanese (ja)' },
  { value: 'ko', label: 'Korean (ko)' },
  { value: 'zh', label: 'Chinese (zh)' },
]

function handleModeChange(mode: TranscriptionMode) {
  transcriptionMode.value = mode
  if (mode === 'cloud') {
    // Switch to default cloud provider if currently on local
    if (['local-whisper', 'remote-whisper'].includes(provider.value)) {
      settingsStore.setProvider('openai')
      // Auto-sync polisher to match (if user has cloud polisher enabled)
      if (polisher.value !== 'none' && polisher.value !== 'ollama') {
        settingsStore.setPolisher('openai')
      }
    }
  } else {
    // Switch to local-whisper if currently on cloud
    if (!['local-whisper', 'remote-whisper'].includes(provider.value)) {
      settingsStore.setProvider('local-whisper')
    }
  }
}

function handleProviderUpdate(newProvider: Provider) {
  settingsStore.setProvider(newProvider)
  // Auto-sync polisher to match provider (if user has cloud polisher enabled)
  if ((newProvider === 'openai' || newProvider === 'mistral') &&
      polisher.value !== 'none' && polisher.value !== 'ollama') {
    settingsStore.setPolisher(newProvider as Polisher)
  }
}

function handleApiKeyUpdate(providerKey: string, value: string) {
  settingsStore.setApiKey(providerKey, value)
}

function handleLocalModeChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Provider
  settingsStore.setProvider(value)
}

function handleLanguageChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  settingsStore.setLanguage(value === '' ? null : value)
}

async function saveSettings() {
  try {
    // Validate OpenAI key format if provided
    const openaiKey = apiKeys.value.openai || ''
    if (openaiKey && !openaiKey.startsWith('sk-')) {
      savingStatus.value = "Invalid OpenAI key format. Keys start with 'sk-'"
      return
    }

    // Validate Groq key format if provided
    const groqKey = apiKeys.value.groq || ''
    if (groqKey && !groqKey.startsWith('gsk_')) {
      savingStatus.value = "Invalid Groq key format. Keys start with 'gsk_'"
      return
    }

    await settingsStore.save()
    savingStatus.value = 'Saved'
    setTimeout(() => savingStatus.value = '', 2000)
  } catch (e) {
    savingStatus.value = 'Failed to save: ' + e
  }
}
</script>

<template>
  <section class="section">
    <header class="section-header">
      <h1>Settings</h1>
      <p>Configure transcription</p>
    </header>

    <div class="section-content">
      <!-- Mode Cards (Cloud/Local) -->
      <ModeCards
        :model-value="transcriptionMode"
        @update:model-value="handleModeChange"
      />

      <!-- Status - Ready -->
      <div v-if="configStatus.ready" class="notice">
        <span class="notice-marker">[*]</span>
        <p>Ready to transcribe · {{ configStatus.message }}</p>
      </div>

      <!-- Cloud Provider Config -->
      <CloudProviderConfig
        v-if="transcriptionMode === 'cloud'"
        :provider="provider"
        :api-keys="apiKeys"
        :show-config-card="!configStatus.ready"
        @update:provider="handleProviderUpdate"
        @update:api-key="handleApiKeyUpdate"
      />

      <!-- Local Whisper Config -->
      <LocalWhisperConfig
        v-if="transcriptionMode === 'local' && provider === 'local-whisper'"
        :show-config-card="!configStatus.ready"
      />

      <!-- Remote Whisper Config -->
      <RemoteWhisperConfig
        v-if="transcriptionMode === 'local' && provider === 'remote-whisper'"
        :show-config-card="!configStatus.ready"
      />

      <!-- Primary Options (always visible) -->
      <div class="primary-options">
        <!-- Local Mode Selection -->
        <div v-if="transcriptionMode === 'local'" class="field-row">
          <label>Mode</label>
          <select
            class="select-input"
            :value="provider"
            @change="handleLocalModeChange"
          >
            <option v-for="opt in localModeOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>

        <!-- Language Hint -->
        <div class="field-row">
          <label>Language</label>
          <select
            class="select-input"
            :value="language ?? ''"
            @change="handleLanguageChange"
          >
            <option v-for="opt in languageOptions" :key="opt.value ?? 'auto'" :value="opt.value ?? ''">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>

      <!-- Advanced Options Toggle -->
      <button class="advanced-toggle" @click="showAdvanced = !showAdvanced">
        <span class="toggle-arrow">{{ showAdvanced ? 'v' : '>' }}</span>
        <span>Advanced options</span>
      </button>

      <!-- Advanced Options Content -->
      <div v-show="showAdvanced" class="advanced-content">
        <PolishingConfig />

        <!-- Ollama Config (only when polisher is ollama) -->
        <template v-if="polisher === 'ollama'">
          <OllamaConfig />
        </template>
      </div>

      <button @click="saveSettings" class="btn btn-secondary save-btn">Save</button>

      <div class="status" :class="{ visible: savingStatus }" role="status" aria-live="polite">{{ savingStatus }}</div>

      <div class="notice">
        <span class="notice-marker">[i]</span>
        <p>Settings stored in ~/.config/whis/settings.json</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* Notice (matches App.vue pattern) */
.notice {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  margin-bottom: 16px;
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

/* Primary Options (always visible) */
.primary-options {
  padding: 16px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 6px;
  display: grid;
  gap: 12px;
  margin-bottom: 8px;
}

/* Advanced Options */
.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 0;
  background: none;
  border: none;
  color: var(--text-weak);
  cursor: pointer;
  font-family: var(--font);
  font-size: 12px;
  width: 100%;
  text-align: left;
}

.advanced-toggle:hover {
  color: var(--text);
}

.advanced-toggle:focus-visible {
  outline: none;
  color: var(--accent);
}

.toggle-arrow {
  font-size: 10px;
  width: 12px;
}

.advanced-content {
  padding: 16px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 6px;
  display: grid;
  gap: 12px;
  margin-bottom: 16px;
}

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

/* Save button spacing */
.save-btn {
  margin-top: 8px;
}

/* Select input */
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
</style>
