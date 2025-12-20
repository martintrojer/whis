import type { UnlistenFn } from '@tauri-apps/api/event'
import type { WhisperModelInfo } from '../types'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { settingsStore } from '../stores/settings'

// Model sizes for display
const MODEL_SIZES: Record<string, string> = {
  tiny: '~75 MB',
  base: '~142 MB',
  small: '~466 MB',
  medium: '~1.5 GB',
}

/**
 * Composable for managing Whisper model downloads and validation.
 * Used in ApiKeyView for local transcription configuration.
 */
export function useWhisperModel() {
  const whisperModelValid = ref(false)
  const availableModels = ref<WhisperModelInfo[]>([])
  const selectedModel = ref('small')
  const downloadingModel = ref(false)
  const downloadStatus = ref('')
  const downloadProgress = ref<{ downloaded: number, total: number } | null>(null)

  let downloadUnlisten: UnlistenFn | null = null

  // Computed properties from store
  const provider = computed(() => settingsStore.state.provider)
  const whisperModelPath = computed(() => settingsStore.state.whisper_model_path)

  // Check if whisper model file exists on disk
  async function checkWhisperModel() {
    if (provider.value === 'local-whisper') {
      try {
        whisperModelValid.value = await invoke<boolean>('is_whisper_model_valid')
      }
      catch {
        whisperModelValid.value = false
      }
    }
  }

  // Fetch available whisper models
  async function loadWhisperModels() {
    try {
      availableModels.value = await invoke<WhisperModelInfo[]>('get_whisper_models')
      // If a model is already installed, select it by default
      const installed = availableModels.value.find(m => m.installed)
      if (installed) {
        selectedModel.value = installed.name
      }
    }
    catch (e) {
      console.error('Failed to load whisper models:', e)
    }
  }

  // Download the selected model
  async function downloadModel() {
    downloadingModel.value = true
    downloadProgress.value = null
    downloadStatus.value = ''

    try {
      // Listen for progress events
      downloadUnlisten = await listen<{ downloaded: number, total: number }>('download-progress', (event) => {
        downloadProgress.value = event.payload
      })

      const path = await invoke<string>('download_whisper_model', { modelName: selectedModel.value })
      settingsStore.setWhisperModelPath(path)
      whisperModelValid.value = true
      downloadStatus.value = 'Model downloaded successfully!'
      // Refresh model list to update installed status
      await loadWhisperModels()
      setTimeout(() => downloadStatus.value = '', 3000)
    }
    catch (e) {
      downloadStatus.value = `Download failed: ${e}`
    }
    finally {
      downloadingModel.value = false
      downloadProgress.value = null
      if (downloadUnlisten) {
        downloadUnlisten()
        downloadUnlisten = null
      }
    }
  }

  // Format download progress for display
  const downloadProgressPercent = computed(() => {
    if (!downloadProgress.value || downloadProgress.value.total === 0)
      return 0
    return Math.round((downloadProgress.value.downloaded / downloadProgress.value.total) * 100)
  })

  const downloadProgressText = computed(() => {
    if (!downloadProgress.value)
      return ''
    const { downloaded, total } = downloadProgress.value
    const downloadedMB = (downloaded / 1_000_000).toFixed(0)
    const totalMB = (total / 1_000_000).toFixed(0)
    return `${downloadedMB} MB / ${totalMB} MB`
  })

  // Check if selected model is installed
  const isSelectedModelInstalled = computed(() => {
    return availableModels.value.find(m => m.name === selectedModel.value)?.installed ?? false
  })

  // Get size for selected model
  const selectedModelSize = computed(() => {
    return MODEL_SIZES[selectedModel.value] ?? ''
  })

  // Setup watchers and lifecycle
  onMounted(() => {
    checkWhisperModel()
    loadWhisperModels()
  })

  watch(provider, checkWhisperModel)
  watch(whisperModelPath, checkWhisperModel)

  // Cleanup download listener on unmount
  onUnmounted(() => {
    if (downloadUnlisten) {
      downloadUnlisten()
      downloadUnlisten = null
    }
  })

  return {
    // State
    whisperModelValid,
    availableModels,
    selectedModel,
    downloadingModel,
    downloadStatus,
    downloadProgress,

    // Computed
    downloadProgressPercent,
    downloadProgressText,
    isSelectedModelInstalled,
    selectedModelSize,

    // Actions
    checkWhisperModel,
    loadWhisperModels,
    downloadModel,

    // Constants
    MODEL_SIZES,
  }
}
