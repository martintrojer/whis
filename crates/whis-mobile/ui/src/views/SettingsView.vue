<script setup lang="ts">
import type { SelectOption } from '../types'
import { computed } from 'vue'
import AppInput from '../components/AppInput.vue'
import AppSelect from '../components/AppSelect.vue'
import { settingsStore } from '../stores/settings'

// Provider options
const providerOptions: SelectOption[] = [
  { value: 'openai', label: 'OpenAI Whisper' },
  { value: 'mistral', label: 'Mistral Voxtral' },
]

// Language options
const languageOptions: SelectOption[] = [
  { value: null, label: 'Auto-detect' },
  { value: 'en', label: 'English' },
  { value: 'de', label: 'German' },
  { value: 'fr', label: 'French' },
  { value: 'es', label: 'Spanish' },
  { value: 'it', label: 'Italian' },
  { value: 'pt', label: 'Portuguese' },
  { value: 'nl', label: 'Dutch' },
  { value: 'pl', label: 'Polish' },
  { value: 'ja', label: 'Japanese' },
  { value: 'zh', label: 'Chinese' },
  { value: 'ko', label: 'Korean' },
]

// Computed bindings to store
const provider = computed({
  get: () => settingsStore.state.provider,
  set: val => settingsStore.setProvider(val),
})

const language = computed({
  get: () => settingsStore.state.language,
  set: val => settingsStore.setLanguage(val),
})

const openaiApiKey = computed({
  get: () => settingsStore.state.openai_api_key ?? '',
  set: val => settingsStore.setOpenaiApiKey(val || null),
})

const mistralApiKey = computed({
  get: () => settingsStore.state.mistral_api_key ?? '',
  set: val => settingsStore.setMistralApiKey(val || null),
})
</script>

<template>
  <div class="settings-view">
    <main class="settings-content">
      <h1 class="page-title">
        Settings
      </h1>
      <div class="field">
        <label>provider</label>
        <AppSelect
          v-model="provider"
          :options="providerOptions"
          aria-label="Select provider"
        />
      </div>

      <div v-if="provider === 'openai'" class="field">
        <label>openai api key</label>
        <AppInput
          v-model="openaiApiKey"
          type="password"
          placeholder="sk-..."
        />
        <span class="hint">
          Get your key at <a href="https://platform.openai.com/api-keys" target="_blank">platform.openai.com</a>
        </span>
      </div>

      <div v-if="provider === 'mistral'" class="field">
        <label>mistral api key</label>
        <AppInput
          v-model="mistralApiKey"
          type="password"
          placeholder="Enter API key"
        />
        <span class="hint">
          Get your key at <a href="https://console.mistral.ai/api-keys" target="_blank">console.mistral.ai</a>
        </span>
      </div>

      <div class="field">
        <label>language</label>
        <AppSelect
          v-model="language"
          :options="languageOptions"
          aria-label="Select language"
        />
        <span class="hint">
          Language of the audio being transcribed
        </span>
      </div>

      <div class="auto-save-notice">
        <span class="notice-marker">[*]</span>
        <span>Settings are saved automatically</span>
      </div>
    </main>
  </div>
</template>

<style scoped>
.settings-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

/* Page Title */
.page-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 8px;
}

/* Content */
.settings-content {
  flex: 1;
  padding: 20px;
  padding-bottom: max(20px, env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 24px;
}

/* Auto-save notice */
.auto-save-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: auto;
  padding-top: 24px;
  font-size: 13px;
  color: var(--text-weak);
}

.notice-marker {
  color: var(--accent);
}
</style>
