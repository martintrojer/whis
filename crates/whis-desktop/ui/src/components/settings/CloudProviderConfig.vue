<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Provider, CloudProviderInfo } from '../../types'

const props = defineProps<{
  provider: Provider
  apiKeys: Record<string, string>
  showConfigCard?: boolean
}>()

const emit = defineEmits<{
  'update:provider': [value: Provider]
  'update:apiKey': [provider: string, value: string]
}>()

const keyMasked = ref<Record<string, boolean>>({
  openai: true,
  mistral: true,
  groq: true,
  deepgram: true,
  elevenlabs: true,
})

// Cloud provider options with metadata
const cloudProviders: CloudProviderInfo[] = [
  { value: 'openai', label: 'OpenAI', desc: 'Industry standard, excellent accuracy', keyUrl: 'https://platform.openai.com/api-keys', placeholder: 'sk-...' },
  { value: 'mistral', label: 'Mistral', desc: 'European AI, multilingual support', keyUrl: 'https://console.mistral.ai/api-keys', placeholder: '...' },
  { value: 'groq', label: 'Groq', desc: 'Ultra-fast inference', keyUrl: 'https://console.groq.com/keys', placeholder: 'gsk_...' },
  { value: 'deepgram', label: 'Deepgram', desc: 'Real-time optimized', keyUrl: 'https://console.deepgram.com', placeholder: '...' },
  { value: 'elevenlabs', label: 'ElevenLabs', desc: 'Best for voice projects', keyUrl: 'https://elevenlabs.io/app/settings/api-keys', placeholder: '...' },
]

const currentProvider = computed((): CloudProviderInfo => {
  const found = cloudProviders.find(p => p.value === props.provider)
  return found ?? cloudProviders[0]!
})

const currentApiKey = computed(() => props.apiKeys[props.provider] || '')
const hasApiKey = computed(() => !!currentApiKey.value)

function handleProviderChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as Provider
  emit('update:provider', value)
}

function handleApiKeyChange(event: Event) {
  const value = (event.target as HTMLInputElement).value
  emit('update:apiKey', props.provider, value)
}
</script>

<template>
  <!-- API Key Configuration Card (shown when key is missing) -->
  <div v-if="showConfigCard && !hasApiKey" class="config-card">
    <div class="notice">
      <span class="notice-marker">[!]</span>
      <p>API key required</p>
    </div>
    <div class="config-section">
      <div class="api-key-input">
        <input
          :type="keyMasked[provider] ? 'password' : 'text'"
          :value="currentApiKey"
          @input="handleApiKeyChange"
          :placeholder="currentProvider.placeholder"
          spellcheck="false"
          autocomplete="off"
          aria-label="API Key"
        />
        <button
          @click="keyMasked[provider] = !keyMasked[provider]"
          class="toggle-btn"
          type="button"
          :aria-pressed="!keyMasked[provider]"
          aria-label="Toggle API key visibility"
        >
          {{ keyMasked[provider] ? 'show' : 'hide' }}
        </button>
      </div>
      <p class="hint">
        Get key at
        <a :href="currentProvider.keyUrl" target="_blank">{{ currentProvider.keyUrl.replace('https://', '') }}</a>
      </p>
    </div>
  </div>

  <!-- Provider Selection Row -->
  <div class="field-row">
    <label>Provider</label>
    <select
      class="select-input"
      :value="provider"
      @change="handleProviderChange"
    >
      <option v-for="p in cloudProviders" :key="p.value" :value="p.value">
        {{ p.label }}
      </option>
    </select>
  </div>
</template>

<style scoped>
.config-card {
  margin-bottom: 16px;
}

.config-card .notice {
  margin-bottom: 0;
  border-bottom-left-radius: 0;
  border-bottom-right-radius: 0;
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

.config-section {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-top: none;
  border-bottom-left-radius: 4px;
  border-bottom-right-radius: 4px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.api-key-input {
  display: flex;
  gap: 8px;
}

.api-key-input input {
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

.api-key-input input::placeholder {
  color: var(--text-weak);
}

.api-key-input input:focus {
  outline: none;
  border-color: var(--accent);
}

.toggle-btn {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 11px;
  color: var(--text-weak);
  cursor: pointer;
  transition: all 0.15s ease;
}

.toggle-btn:hover {
  border-color: var(--text-weak);
  color: var(--text);
}

.toggle-btn:focus-visible {
  outline: none;
  border-color: var(--accent);
}

.hint {
  font-size: 11px;
  color: var(--text-weak);
  margin: 0;
}

.hint a {
  color: var(--accent);
  text-decoration: none;
}

.hint a:hover {
  text-decoration: underline;
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

.field-row > select {
  flex: 1;
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
</style>
