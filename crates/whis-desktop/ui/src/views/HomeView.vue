<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { settingsStore } from '../stores/settings'
import type { StatusResponse } from '../types'

const status = ref<StatusResponse>({ state: 'Idle', config_valid: false })
const error = ref<string | null>(null)
const polishWarning = ref<string | null>(null)
const isPolishing = ref(false)
let pollInterval: number | null = null
let unlistenPolishWarning: UnlistenFn | null = null
let unlistenPolishStarted: UnlistenFn | null = null
let unlistenTranscriptionComplete: UnlistenFn | null = null

// Configuration readiness state (proactive checks)
const configReadiness = ref<{
  transcriptionReady: boolean
  transcriptionError: string | null
  polishingReady: boolean
  polishingError: string | null
  checking: boolean
}>({
  transcriptionReady: true,
  transcriptionError: null,
  polishingReady: true,
  polishingError: null,
  checking: false
})

const buttonText = computed(() => {
  switch (status.value.state) {
    case 'Idle': return 'Start Recording'
    case 'Recording': return 'Stop Recording'
    case 'Transcribing': return 'Transcribing...'
  }
})

// Configuration summary for status display (compact single line)
const configSummary = computed(() => {
  const { provider, language, polisher, active_preset } = settingsStore.state

  // Mode + Provider
  let mode = 'Cloud'
  let providerName: string = provider
  if (provider === 'local-whisper') {
    mode = 'Local'
    providerName = 'Whisper'
  } else if (provider === 'remote-whisper') {
    mode = 'Remote'
    providerName = 'Whisper'
  } else {
    // Capitalize cloud provider names
    providerName = provider.charAt(0).toUpperCase() + provider.slice(1)
  }

  // Language: show code or omit if auto-detect
  const lang = language ? language.toUpperCase() : null

  // Polishing status: show preset name if active, "Polishing ON" if enabled but no preset, omit if off
  let polishStatus: string | null = null
  if (polisher !== 'none') {
    polishStatus = active_preset || 'Polishing'
  }

  return { mode, provider: providerName, lang, polishStatus }
})

const canRecord = computed(() => {
  return status.value.config_valid &&
         status.value.state !== 'Transcribing' &&
         configReadiness.value.transcriptionReady
})

// Check configuration readiness (proactive check for better UX)
async function checkConfigReadiness() {
  const { provider, polisher, api_keys, whisper_model_path, remote_whisper_url, ollama_url } = settingsStore.state

  configReadiness.value.checking = true
  try {
    const result = await invoke<{
      transcription_ready: boolean
      transcription_error: string | null
      polishing_ready: boolean
      polishing_error: string | null
    }>('check_config_readiness', {
      provider,
      polisher,
      apiKeys: api_keys,
      whisperModelPath: whisper_model_path,
      remoteWhisperUrl: remote_whisper_url,
      ollamaUrl: ollama_url
    })
    configReadiness.value = {
      transcriptionReady: result.transcription_ready,
      transcriptionError: result.transcription_error,
      polishingReady: result.polishing_ready,
      polishingError: result.polishing_error,
      checking: false
    }
  } catch (e) {
    console.error('Failed to check config readiness:', e)
    configReadiness.value.checking = false
  }
}

// Watch for settings changes to re-check readiness
watch(
  () => [
    settingsStore.state.provider,
    settingsStore.state.polisher,
    settingsStore.state.api_keys,
    settingsStore.state.whisper_model_path,
    settingsStore.state.ollama_url
  ],
  () => checkConfigReadiness(),
  { deep: true }
)

const displayShortcut = computed(() => {
  const portalShortcut = settingsStore.state.portalShortcut
  const currentShortcut = settingsStore.state.shortcut

  if (portalShortcut) {
    let shortcut = portalShortcut
    shortcut = shortcut
      .replace(/<Control>/gi, 'Ctrl+')
      .replace(/<Shift>/gi, 'Shift+')
      .replace(/<Alt>/gi, 'Alt+')
      .replace(/<Super>/gi, 'Super+')
    shortcut = shortcut.replace(/\+$/, '')
    const parts = shortcut.split('+')
    if (parts.length > 0 && parts[parts.length - 1]) {
      parts[parts.length - 1] = parts[parts.length - 1]!.toUpperCase()
    }
    return parts.join('+')
  }
  return currentShortcut || null
})

async function fetchStatus() {
  try {
    status.value = await invoke<StatusResponse>('get_status');
    error.value = null;
  } catch (e) {
    console.error('Failed to get status:', e);
  }
}

async function toggleRecording() {
  if (!canRecord.value) return;

  try {
    error.value = null;
    await invoke('toggle_recording');
    await fetchStatus();
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(async () => {
  fetchStatus();
  checkConfigReadiness();
  pollInterval = window.setInterval(fetchStatus, 500);

  // Listen for polish events
  unlistenPolishWarning = await listen<string>('polish-warning', (event) => {
    polishWarning.value = event.payload;
    // Auto-dismiss after 8 seconds
    setTimeout(() => {
      polishWarning.value = null;
    }, 8000);
  });

  unlistenPolishStarted = await listen('polish-started', () => {
    isPolishing.value = true;
  });

  unlistenTranscriptionComplete = await listen('transcription-complete', () => {
    isPolishing.value = false;
  });
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
  unlistenPolishWarning?.();
  unlistenPolishStarted?.();
  unlistenTranscriptionComplete?.();
});
</script>

<template>
  <section class="section">
    <header class="section-header">
      <h1>Home</h1>
      <p>Your voice, piped to clipboard</p>
    </header>

    <div class="section-content">
      <!-- Recording action -->
      <div class="record-action">
        <button
          class="btn btn-secondary"
          :class="{ recording: status.state === 'Recording', transcribing: status.state === 'Transcribing' }"
          :disabled="!canRecord"
          @click="toggleRecording"
        >
          <span class="record-indicator"></span>
          <span>{{ buttonText }}</span>
        </button>

        <!-- Shortcut hint - shown inline when available -->
        <span v-if="displayShortcut && status.state === 'Idle'" class="shortcut-hint">
          or press <kbd>{{ displayShortcut }}</kbd>
        </span>

        <!-- Config summary - compact single line status -->
        <span v-if="status.config_valid && status.state === 'Idle'" class="config-summary">
          <span :class="{ 'config-error': !configReadiness.transcriptionReady }">{{ configSummary.mode }} · {{ configSummary.provider }}</span><span v-if="configSummary.lang"> · {{ configSummary.lang }}</span><span v-if="configSummary.polishStatus" :class="{ 'config-warning': !configReadiness.polishingReady }"> · {{ configSummary.polishStatus }}</span>
        </span>

        <!-- State hints (announced to screen readers) -->
        <span role="status" aria-live="polite" class="state-hints">
          <span v-if="status.state === 'Recording'" class="state-hint recording">
            speak now...
          </span>
          <span v-else-if="isPolishing" class="state-hint polishing">
            polishing...
          </span>
          <span v-else-if="status.state === 'Transcribing'" class="state-hint">
            processing audio...
          </span>
        </span>
      </div>

      <!-- Error message -->
      <p v-if="error" class="error-msg">{{ error }}</p>

      <!-- Polish warning (runtime) -->
      <div v-if="polishWarning" class="warning-msg">
        <strong>Polishing skipped:</strong> {{ polishWarning }}
      </div>

      <!-- Transcription not ready (blocking) -->
      <div v-if="!configReadiness.transcriptionReady && status.config_valid" class="config-notice error">
        <span class="notice-marker">[!]</span>
        <div>
          <p>{{ configReadiness.transcriptionError }}</p>
          <router-link to="/settings">Configure →</router-link>
        </div>
      </div>

      <!-- Polishing not ready (non-blocking warning) -->
      <div v-else-if="!configReadiness.polishingReady && settingsStore.state.polisher !== 'none' && status.config_valid" class="config-notice warning">
        <span class="notice-marker">[!]</span>
        <div>
          <p>Polishing unavailable: {{ configReadiness.polishingError }}</p>
          <router-link to="/settings">Configure →</router-link>
        </div>
      </div>

      <!-- Only show notice when something needs attention -->
      <div v-if="!status.config_valid" class="notice">
        <span class="notice-marker">[!]</span>
        <p>Configure your provider in <strong>settings</strong> to start transcribing.</p>
      </div>

      <div v-else-if="!displayShortcut" class="notice">
        <span class="notice-marker">[*]</span>
        <p>Configure a global <strong>shortcut</strong> to record hands-free.</p>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* Recording action group */
.record-action {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* Button needs inline-flex for indicator */
.btn.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

/* Recording state - red */
.btn.btn-secondary.recording {
  background: var(--recording);
  border-color: var(--recording);
  color: white;
}

.btn.btn-secondary.recording:hover:not(:disabled) {
  background: #ff6666;
  border-color: #ff6666;
}

/* Transcribing state */
.btn.btn-secondary.transcribing {
  background: var(--bg-weak);
  border-color: var(--border);
  color: var(--text-weak);
  cursor: wait;
}

/* Status indicator dot inside button */
.record-indicator {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  opacity: 0.5;
}

.btn.btn-secondary.recording .record-indicator {
  opacity: 1;
  animation: pulse 1s ease-in-out infinite;
}

.btn.btn-secondary.transcribing .record-indicator {
  background: var(--accent);
  opacity: 1;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

/* Shortcut hint */
.shortcut-hint {
  font-size: 11px;
  color: var(--text-weak);
}

/* Config summary */
.config-summary {
  font-size: 10px;
  color: var(--text-weak);
  opacity: 0.7;
}

.shortcut-hint kbd {
  display: inline-block;
  padding: 2px 6px;
  font-family: var(--font);
  font-size: 10px;
  color: var(--accent);
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 3px;
}

/* State hints */
.state-hint {
  font-size: 11px;
  color: var(--text-weak);
  font-style: italic;
}

.state-hint.recording {
  color: var(--recording);
}

.state-hint.polishing {
  color: var(--accent);
}

/* Error message */
.error-msg {
  font-size: 12px;
  color: var(--recording);
  padding: 8px 12px;
  background: rgba(255, 68, 68, 0.1);
  border: 1px solid rgba(255, 68, 68, 0.3);
  border-radius: 4px;
}

/* Warning message (for polish warnings) */
.warning-msg {
  font-size: 12px;
  color: var(--text);
  padding: 8px 12px;
  background: rgba(255, 180, 68, 0.1);
  border: 1px solid rgba(255, 180, 68, 0.3);
  border-radius: 4px;
}

.warning-msg strong {
  color: #ffb444;
}

/* Notice overrides */
.notice strong {
  color: var(--text-strong);
}

/* Config readiness warnings in summary */
.config-error {
  color: #f87171;
}

.config-warning {
  opacity: 0.5;
}

/* Config readiness notice cards */
.config-notice {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-size: 12px;
}

.config-notice p {
  margin: 0 0 4px 0;
  color: var(--text);
}

.config-notice a {
  color: var(--accent);
  text-decoration: none;
  font-size: 11px;
}

.config-notice a:hover {
  text-decoration: underline;
}

.config-notice.error .notice-marker {
  color: #f87171;
}

.config-notice.warning .notice-marker {
  color: var(--accent);
}
</style>
