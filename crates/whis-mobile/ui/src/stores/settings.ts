import type { Provider } from '../types'
import { Store } from '@tauri-apps/plugin-store'
import { reactive, readonly } from 'vue'

// Internal mutable state for reactive UI
const state = reactive({
  provider: 'openai' as Provider,
  language: null as string | null,
  openai_api_key: null as string | null,
  mistral_api_key: null as string | null,
  loaded: false,
})

// Tauri store instance (initialized lazily)
let store: Store | null = null

async function getStore(): Promise<Store> {
  if (!store) {
    store = await Store.load('settings.json', {
      autoSave: 500,
      defaults: {
        provider: 'openai',
        language: null,
        openai_api_key: null,
        mistral_api_key: null,
      },
    })
  }
  return store
}

// Actions
async function initialize() {
  try {
    const s = await getStore()

    // Load values from store
    state.provider = (await s.get<Provider>('provider')) || 'openai'
    state.language = (await s.get<string | null>('language')) ?? null
    state.openai_api_key = (await s.get<string | null>('openai_api_key')) ?? null
    state.mistral_api_key = (await s.get<string | null>('mistral_api_key')) ?? null
    state.loaded = true
  }
  catch (e) {
    console.error('Failed to load settings:', e)
    state.loaded = true // Mark as loaded even on error to unblock UI
  }
}

async function setProvider(value: Provider) {
  state.provider = value
  const s = await getStore()
  await s.set('provider', value)
}

async function setLanguage(value: string | null) {
  state.language = value
  const s = await getStore()
  await s.set('language', value)
}

async function setOpenaiApiKey(value: string | null) {
  state.openai_api_key = value
  const s = await getStore()
  await s.set('openai_api_key', value)
}

async function setMistralApiKey(value: string | null) {
  state.mistral_api_key = value
  const s = await getStore()
  await s.set('mistral_api_key', value)
}

// Export reactive state and actions
export const settingsStore = {
  // Readonly state for reading
  state: readonly(state),

  // Actions
  initialize,

  // Setters
  setProvider,
  setLanguage,
  setOpenaiApiKey,
  setMistralApiKey,
}
