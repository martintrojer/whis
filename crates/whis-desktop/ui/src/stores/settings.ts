import { reactive, readonly, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Settings, BackendInfo, Provider, Polisher } from '../types'

// Simple debounce utility
function debounce<T extends (...args: unknown[]) => unknown>(fn: T, ms: number) {
  let timeoutId: ReturnType<typeof setTimeout> | null = null
  return (...args: Parameters<T>) => {
    if (timeoutId) clearTimeout(timeoutId)
    timeoutId = setTimeout(() => fn(...args), ms)
  }
}

// Default settings values
const defaultSettings: Settings = {
  shortcut: 'Ctrl+Shift+R',
  provider: 'openai',
  language: null,
  api_keys: {},
  whisper_model_path: null,
  remote_whisper_url: null,
  polisher: 'none',
  ollama_url: null,
  ollama_model: null,
  polish_prompt: null,
  active_preset: null,
}

// Internal mutable state
const state = reactive({
  // Settings
  ...defaultSettings,

  // Backend info
  backendInfo: null as BackendInfo | null,
  portalShortcut: null as string | null,
  portalBindError: null as string | null,

  // Loading state
  loaded: false,
})

// Debounced auto-save (500ms delay)
const debouncedSave = debounce(async () => {
  try {
    await invoke<{ needs_restart: boolean }>('save_settings', {
      settings: {
        shortcut: state.shortcut,
        provider: state.provider,
        language: state.language,
        api_keys: state.api_keys,
        whisper_model_path: state.whisper_model_path,
        remote_whisper_url: state.remote_whisper_url,
        polisher: state.polisher,
        ollama_url: state.ollama_url,
        ollama_model: state.ollama_model,
        polish_prompt: state.polish_prompt,
        active_preset: state.active_preset,
      },
    })
  } catch (e) {
    console.error('Auto-save failed:', e)
  }
}, 500)

// Watch settings and auto-save on change
watch(
  () => [
    state.provider,
    state.language,
    state.api_keys,
    state.whisper_model_path,
    state.remote_whisper_url,
    state.polisher,
    state.ollama_url,
    state.ollama_model,
    state.polish_prompt,
    state.active_preset,
  ],
  () => {
    if (state.loaded) debouncedSave()
  },
  { deep: true }
)

// Actions
async function load() {
  try {
    const settings = await invoke<Settings>('get_settings')
    state.shortcut = settings.shortcut
    state.provider = settings.provider || 'openai'
    state.language = settings.language
    state.api_keys = settings.api_keys || {}
    state.whisper_model_path = settings.whisper_model_path
    state.remote_whisper_url = settings.remote_whisper_url
    state.polisher = settings.polisher || 'none'
    state.ollama_url = settings.ollama_url
    state.ollama_model = settings.ollama_model
    state.polish_prompt = settings.polish_prompt
    state.active_preset = settings.active_preset
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
}

async function save(): Promise<boolean> {
  try {
    const result = await invoke<{ needs_restart: boolean }>('save_settings', {
      settings: {
        shortcut: state.shortcut,
        provider: state.provider,
        language: state.language,
        api_keys: state.api_keys,
        whisper_model_path: state.whisper_model_path,
        remote_whisper_url: state.remote_whisper_url,
        polisher: state.polisher,
        ollama_url: state.ollama_url,
        ollama_model: state.ollama_model,
        polish_prompt: state.polish_prompt,
        active_preset: state.active_preset,
      },
    })
    return result.needs_restart
  } catch (e) {
    console.error('Failed to save settings:', e)
    throw e
  }
}

async function loadBackendInfo() {
  try {
    state.backendInfo = await invoke<BackendInfo>('shortcut_backend')

    // For portal backend, fetch actual binding and any errors
    if (state.backendInfo?.backend === 'PortalGlobalShortcuts') {
      state.portalShortcut = await invoke<string | null>('portal_shortcut')
      state.portalBindError = await invoke<string | null>('portal_bind_error')
    }
  } catch (e) {
    console.error('Failed to get backend info:', e)
  }
}

async function initialize() {
  await loadBackendInfo()
  await load()
  state.loaded = true
}

// Setters for individual fields (for v-model binding)
function setProvider(value: Provider) {
  state.provider = value
}

function setLanguage(value: string | null) {
  state.language = value
}

function setApiKey(provider: string, key: string) {
  state.api_keys = { ...state.api_keys, [provider]: key }
}

function setWhisperModelPath(value: string | null) {
  state.whisper_model_path = value
}

function setRemoteWhisperUrl(value: string | null) {
  state.remote_whisper_url = value
}

function setPolisher(value: Polisher) {
  state.polisher = value
}

function setOllamaUrl(value: string | null) {
  state.ollama_url = value
}

function setOllamaModel(value: string | null) {
  state.ollama_model = value
}

function setPolishPrompt(value: string | null) {
  state.polish_prompt = value
}

function setShortcut(value: string) {
  state.shortcut = value
}

function setPortalShortcut(value: string | null) {
  state.portalShortcut = value
}

// Export reactive state and actions
export const settingsStore = {
  // Readonly state for reading (prevents accidental mutation)
  state: readonly(state),

  // Mutable state for v-model (use sparingly)
  mutableState: state,

  // Actions
  load,
  save,
  initialize,

  // Setters
  setProvider,
  setLanguage,
  setApiKey,
  setWhisperModelPath,
  setRemoteWhisperUrl,
  setPolisher,
  setOllamaUrl,
  setOllamaModel,
  setPolishPrompt,
  setShortcut,
  setPortalShortcut,
}
