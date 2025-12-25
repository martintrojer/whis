<script setup lang="ts">
import type { Provider, SelectOption } from '../../types'
import { computed, ref } from 'vue'
import { useWhisperModel } from '../../composables/useWhisperModel'
import { settingsStore } from '../../stores/settings'
import AppSelect from '../AppSelect.vue'

const props = defineProps<{
  showConfigCard?: boolean
  provider?: Provider
}>()

const showPathInput = ref(false)

const {
  availableModels,
  selectedModel,
  downloadingModel,
  downloadStatus,
  downloadProgress,
  downloadProgressPercent,
  downloadProgressText,
  isSelectedModelInstalled,
  selectedModelSize,
  downloadModel,
  MODEL_SIZES,
} = useWhisperModel()

const whisperModelPath = settingsStore.state.whisper_model_path
const parakeetModelPath = settingsStore.state.parakeet_model_path

// Local engine options
const localEngineOptions: SelectOption[] = [
  { value: 'local-parakeet', label: 'Parakeet V3 (Recommended)' },
  { value: 'local-whisper', label: 'Whisper' },
]

// Whether we're using Parakeet
const isParakeet = computed(() => props.provider === 'local-parakeet')

function handleEngineChange(value: string | null) {
  if (value === 'local-whisper' || value === 'local-parakeet') {
    settingsStore.setProvider(value as Provider)
  }
}

// Convert available models to SelectOption format
const modelOptions = computed<SelectOption[]>(() =>
  availableModels.value.map(model => ({
    value: model.name,
    label: `${model.name}${model.installed ? ' [installed]' : ''} - ${MODEL_SIZES[model.name]}`,
  })),
)

function handleModelChange(value: string | null) {
  if (value)
    selectedModel.value = value
}
</script>

<template>
  <!-- Model Configuration Card -->
  <div v-if="showConfigCard" class="config-card">
    <!-- Engine selector -->
    <div class="engine-selector">
      <AppSelect
        :model-value="provider ?? 'local-parakeet'"
        :options="localEngineOptions"
        @update:model-value="handleEngineChange"
      />
    </div>

    <!-- Parakeet config (simplified - just shows status) -->
    <template v-if="isParakeet">
      <p class="hint">
        Parakeet V3: Fast and accurate local transcription (~478 MB)
      </p>
      <button class="path-toggle" type="button" @click="showPathInput = !showPathInput">
        <span class="toggle-indicator">{{ showPathInput ? 'v' : '>' }}</span>
        <span>specify model path</span>
      </button>
      <input
        v-show="showPathInput"
        type="text"
        class="text-input"
        :value="parakeetModelPath || ''"
        placeholder="/path/to/parakeet-model-dir"
        spellcheck="false"
        aria-label="Custom Parakeet model path"
        @input="settingsStore.setParakeetModelPath(($event.target as HTMLInputElement).value || null)"
      >
    </template>

    <!-- Whisper config (existing model selection) -->
    <template v-else>
      <div class="model-selector">
        <AppSelect
          :model-value="selectedModel"
          :options="modelOptions"
          :disabled="downloadingModel"
          @update:model-value="handleModelChange"
        />
        <button
          class="btn-primary"
          :disabled="downloadingModel || isSelectedModelInstalled"
          @click="downloadModel"
        >
          {{ downloadingModel ? `${downloadProgressPercent}%` : isSelectedModelInstalled ? 'Installed' : 'Download' }}
        </button>
      </div>
      <p v-if="downloadProgress" class="hint">
        {{ downloadProgressText }}
      </p>
      <p v-else-if="downloadStatus" class="hint" :class="{ error: downloadStatus.includes('failed'), success: downloadStatus.includes('successfully') }">
        {{ downloadStatus }}
      </p>
      <p v-else class="hint">
        {{ selectedModel === 'small' ? 'Recommended for most users' : selectedModelSize }}
      </p>
      <button class="path-toggle" type="button" @click="showPathInput = !showPathInput">
        <span class="toggle-indicator">{{ showPathInput ? 'v' : '>' }}</span>
        <span>or specify path</span>
      </button>
      <input
        v-show="showPathInput"
        type="text"
        class="text-input"
        :value="whisperModelPath || ''"
        placeholder="/path/to/model.bin"
        spellcheck="false"
        aria-label="Custom Whisper model path"
        @input="settingsStore.setWhisperModelPath(($event.target as HTMLInputElement).value || null)"
      >
    </template>
  </div>
</template>

<style scoped>
.config-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
}

.engine-selector {
  margin-bottom: 4px;
}

.model-selector {
  display: flex;
  gap: 8px;
}

.model-selector :deep(.custom-select) {
  flex: 1;
  min-width: 0;
}

.model-selector .btn-primary {
  flex-shrink: 0;
  min-width: 80px;
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

.path-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 0;
  background: none;
  border: none;
  color: var(--text-weak);
  cursor: pointer;
  font-family: var(--font);
  font-size: 11px;
  margin-top: 4px;
}

.path-toggle:hover {
  color: var(--text);
}

.path-toggle .toggle-indicator {
  font-size: 10px;
  width: 10px;
}

.text-input {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  transition: border-color 0.15s ease;
  min-width: 0;
  width: 100%;
}

.text-input:focus {
  outline: none;
  border-color: var(--accent);
}

.text-input::placeholder {
  color: var(--text-weak);
}
</style>
