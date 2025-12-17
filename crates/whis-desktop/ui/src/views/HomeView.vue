<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
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

const buttonText = computed(() => {
  switch (status.value.state) {
    case 'Idle': return 'Start Recording'
    case 'Recording': return 'Stop Recording'
    case 'Transcribing': return 'Transcribing...'
  }
})

const canRecord = computed(() => {
  return status.value.config_valid && status.value.state !== 'Transcribing'
})

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

      <!-- Polish warning -->
      <div v-if="polishWarning" class="warning-msg">
        <strong>Polishing skipped:</strong> {{ polishWarning }}
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
</style>
