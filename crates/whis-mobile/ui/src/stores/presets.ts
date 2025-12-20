import type { PresetDetails, PresetInfo } from '../types'
import { invoke } from '@tauri-apps/api/core'
import { reactive, readonly } from 'vue'

interface PresetsState {
  presets: PresetInfo[]
  loading: boolean
  error: string | null
  selectedPreset: PresetDetails | null
  loadingDetails: boolean
}

const state = reactive<PresetsState>({
  presets: [],
  loading: false,
  error: null,
  selectedPreset: null,
  loadingDetails: false,
})

export const presetsStore = {
  state: readonly(state),

  async loadPresets() {
    state.loading = true
    state.error = null
    try {
      state.presets = await invoke<PresetInfo[]>('list_presets')
    }
    catch (e) {
      state.error = String(e)
    }
    finally {
      state.loading = false
    }
  },

  async loadPresetDetails(name: string) {
    state.loadingDetails = true
    try {
      state.selectedPreset = await invoke<PresetDetails>('get_preset_details', { name })
    }
    catch (e) {
      console.error('Failed to load preset details:', e)
      state.selectedPreset = null
    }
    finally {
      state.loadingDetails = false
    }
  },

  async setActivePreset(name: string | null) {
    try {
      await invoke('set_active_preset', { name })
      // Refresh the list to update is_active flags
      await this.loadPresets()
    }
    catch (e) {
      console.error('Failed to set active preset:', e)
    }
  },

  clearSelectedPreset() {
    state.selectedPreset = null
  },
}
