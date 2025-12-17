<script setup lang="ts">
import { ref } from 'vue'
import { useWhisperModel } from '../../composables'
import { settingsStore } from '../../stores/settings'

defineProps<{
  showConfigCard?: boolean
}>()

const showPathInput = ref(false)

const {
  whisperModelValid,
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
</script>

<template>
  <!-- Model Configuration Card (shown when model is not valid) -->
  <div v-if="showConfigCard && !whisperModelValid" class="config-card">
    <div class="notice">
      <span class="notice-marker">[!]</span>
      <p>Model required</p>
    </div>
    <div class="config-section">
      <!-- Model selection -->
      <div class="model-selector">
        <select
          v-model="selectedModel"
          class="select-input"
          :disabled="downloadingModel"
        >
          <option v-for="model in availableModels" :key="model.name" :value="model.name">
            {{ model.name }} {{ model.installed ? '[installed]' : '' }} - {{ MODEL_SIZES[model.name] }}
          </option>
        </select>
        <button
          class="btn-primary"
          @click="downloadModel"
          :disabled="downloadingModel || isSelectedModelInstalled"
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
      <p v-else class="hint">{{ selectedModel === 'small' ? 'Recommended for most users' : selectedModelSize }}</p>
      <button class="path-toggle" @click="showPathInput = !showPathInput" type="button">
        <span class="toggle-indicator">{{ showPathInput ? 'v' : '>' }}</span>
        <span>or specify path</span>
      </button>
      <input
        v-show="showPathInput"
        type="text"
        class="text-input"
        :value="whisperModelPath || ''"
        @input="settingsStore.setWhisperModelPath(($event.target as HTMLInputElement).value || null)"
        placeholder="/path/to/model.bin"
        spellcheck="false"
        aria-label="Custom Whisper model path"
      />
    </div>
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

.model-selector {
  display: flex;
  gap: 8px;
}

.model-selector .select-input {
  flex: 1;
  min-width: 0;
}

.model-selector .btn-primary {
  flex-shrink: 0;
  min-width: 80px;
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
