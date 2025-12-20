<script setup lang="ts">
import type { StatusResponse } from '../types'
import { invoke } from '@tauri-apps/api/core'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { settingsStore } from '../stores/settings'

const router = useRouter()

// State
const status = ref<StatusResponse>({ state: 'Idle', config_valid: false })
const error = ref<string | null>(null)
const lastTranscription = ref<string | null>(null)
const showCopied = ref(false)

let pollInterval: number | null = null

const buttonText = computed(() => {
  switch (status.value.state) {
    case 'Idle': return 'Tap to Record'
    case 'Recording': return 'Tap to Stop'
    case 'Transcribing': return 'Transcribing...'
    default: return 'Tap to Record'
  }
})

const canRecord = computed(() => {
  return status.value.config_valid && status.value.state !== 'Transcribing'
})

async function fetchStatus() {
  try {
    status.value = await invoke<StatusResponse>('get_status')
    error.value = null
  }
  catch (e) {
    console.error('Failed to get status:', e)
  }
}

async function toggleRecording() {
  if (!canRecord.value) {
    if (!status.value.config_valid) {
      router.push('/settings')
    }
    return
  }

  try {
    error.value = null

    if (status.value.state === 'Idle') {
      await invoke('start_recording')
    }
    else if (status.value.state === 'Recording') {
      const text = await invoke<string>('stop_recording')
      lastTranscription.value = text
      showCopied.value = true
      setTimeout(() => showCopied.value = false, 2000)
    }

    await fetchStatus()
  }
  catch (e) {
    error.value = String(e)
    await fetchStatus()
  }
}

onMounted(async () => {
  await settingsStore.initialize()
  await fetchStatus()
  pollInterval = window.setInterval(fetchStatus, 500)
})

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval)
  }
})
</script>

<template>
  <div class="home-view">
    <main class="content">
      <!-- Record Button -->
      <button
        class="record-btn"
        :class="{
          recording: status.state === 'Recording',
          transcribing: status.state === 'Transcribing',
          disabled: !canRecord && status.config_valid,
        }"
        @click="toggleRecording"
      >
        <div class="record-circle">
          <div v-if="status.state === 'Recording'" class="pulse-ring" />
          <svg v-if="status.state === 'Idle'" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 14c1.66 0 3-1.34 3-3V5c0-1.66-1.34-3-3-3S9 3.34 9 5v6c0 1.66 1.34 3 3 3z" />
            <path d="M17 11c0 2.76-2.24 5-5 5s-5-2.24-5-5H5c0 3.53 2.61 6.43 6 6.92V21h2v-3.08c3.39-.49 6-3.39 6-6.92h-2z" />
          </svg>
          <svg v-else-if="status.state === 'Recording'" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" rx="2" />
          </svg>
          <div v-else class="spinner" />
        </div>
      </button>

      <p class="status-text">
        {{ buttonText }}
      </p>

      <!-- Copied Toast -->
      <div v-if="showCopied" class="toast">
        Copied to clipboard!
      </div>

      <!-- Error -->
      <p v-if="error" class="error">
        {{ error }}
      </p>

      <!-- Setup Hint -->
      <p v-if="!status.config_valid" class="setup-hint" @click="router.push('/settings')">
        Tap to configure API key
      </p>

      <!-- Last Transcription Preview -->
      <div v-if="lastTranscription && !error" class="preview">
        <p>{{ lastTranscription.substring(0, 100) }}{{ lastTranscription.length > 100 ? '...' : '' }}</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.home-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 100%;
}

/* Content */
.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 20px;
  padding-bottom: max(20px, env(safe-area-inset-bottom));
  gap: 24px;
}

/* Record Button */
.record-btn {
  width: 160px;
  height: 160px;
  border-radius: 50%;
  border: none;
  background: var(--bg-weak);
  cursor: pointer;
  position: relative;
  transition: transform 0.2s, background 0.2s;
}

.record-btn:active {
  transform: scale(0.95);
}

.record-btn.recording {
  background: var(--recording);
}

.record-btn.transcribing {
  background: var(--bg-weak);
  cursor: wait;
}

.record-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.record-circle {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.record-circle svg {
  width: 64px;
  height: 64px;
  color: var(--text);
}

.recording .record-circle svg {
  color: white;
}

/* Pulse animation */
.pulse-ring {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: 50%;
  border: 3px solid white;
  animation: pulse 1.5s ease-out infinite;
}

/* Spinner */
.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid var(--bg);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

/* Status */
.status-text {
  font-size: 18px;
  color: var(--text-weak);
}

/* Toast */
.toast {
  position: fixed;
  bottom: 100px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--accent);
  color: var(--text-inverted);
  padding: 12px 24px;
  border-radius: var(--radius);
  font-weight: 500;
  animation: fadeIn 0.2s ease;
}

/* Error */
.error {
  color: var(--recording);
  text-align: center;
  padding: 12px;
  background: rgba(255, 68, 68, 0.1);
  border-radius: var(--radius);
  max-width: 300px;
}

/* Setup Hint */
.setup-hint {
  color: var(--accent);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 2px;
}

/* Preview */
.preview {
  max-width: 300px;
  padding: 16px;
  background: var(--bg-weak);
  border-radius: var(--radius);
  font-size: 14px;
  color: var(--text-weak);
  text-align: center;
}
</style>
