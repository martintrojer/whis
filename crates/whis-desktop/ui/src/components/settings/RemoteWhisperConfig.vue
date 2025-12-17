<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { settingsStore } from '../../stores/settings'

defineProps<{
  showConfigCard?: boolean
}>()

const testingConnection = ref(false)
const connectionStatus = ref('')

const remoteWhisperUrl = settingsStore.state.remote_whisper_url

async function testConnection() {
  const url = settingsStore.state.remote_whisper_url
  if (!url) {
    connectionStatus.value = 'Enter a URL first'
    return
  }
  testingConnection.value = true
  connectionStatus.value = 'Testing...'
  try {
    await invoke<boolean>('test_remote_whisper', { url })
    connectionStatus.value = 'Connected!'
    setTimeout(() => connectionStatus.value = '', 3000)
  } catch (e) {
    connectionStatus.value = '' + e
  } finally {
    testingConnection.value = false
  }
}
</script>

<template>
  <!-- Remote URL Configuration Card (shown when URL is missing) -->
  <div v-if="showConfigCard && !remoteWhisperUrl" class="config-card">
    <div class="notice">
      <span class="notice-marker">[!]</span>
      <p>Server URL required</p>
    </div>
    <div class="config-section">
      <div class="url-config">
        <input
          type="text"
          :value="remoteWhisperUrl || ''"
          @input="settingsStore.setRemoteWhisperUrl(($event.target as HTMLInputElement).value || null)"
          placeholder="http://localhost:8765"
          spellcheck="false"
          aria-label="Remote Whisper server URL"
        />
        <button
          class="btn-secondary"
          @click="testConnection"
          :disabled="testingConnection"
        >
          {{ testingConnection ? 'Testing...' : 'Test' }}
        </button>
      </div>
      <p v-if="connectionStatus" class="hint" :class="{ success: connectionStatus === 'Connected!', error: !connectionStatus.includes('Connected') && connectionStatus !== 'Testing...' }" role="status" aria-live="polite">
        {{ connectionStatus }}
      </p>
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

.url-config {
  display: flex;
  gap: 8px;
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

.btn-secondary {
  padding: 10px 16px;
  background: transparent;
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn-secondary:hover:not(:disabled) {
  border-color: var(--text-weak);
}

.btn-secondary:disabled {
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
</style>
