// Generic select option for dropdowns
export interface SelectOption<T = string | null> {
  value: T
  label: string
  disabled?: boolean
}

// Mobile supports subset of providers
export type Provider = 'openai' | 'mistral'

// Settings keys used by Tauri Store plugin
export interface SettingsKeys {
  provider: Provider
  language: string | null
  openai_api_key: string | null
  mistral_api_key: string | null
}

// Status response from backend
export interface StatusResponse {
  state: 'Idle' | 'Recording' | 'Transcribing'
  config_valid: boolean
}

// Preset info from backend
export interface PresetInfo {
  name: string
  description: string
  is_builtin: boolean
  is_active: boolean
}

// Full preset details from backend
export interface PresetDetails {
  name: string
  description: string
  prompt: string
  is_builtin: boolean
}
