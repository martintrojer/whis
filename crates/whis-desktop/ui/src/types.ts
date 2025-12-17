// Generic select option for dropdowns
export interface SelectOption<T = string | null> {
  value: T
  label: string
  disabled?: boolean
}

// Transcription providers
export type Provider =
  | 'openai'
  | 'mistral'
  | 'groq'
  | 'deepgram'
  | 'elevenlabs'
  | 'local-whisper'
  | 'remote-whisper'

// Text polishing providers
export type Polisher = 'none' | 'openai' | 'mistral' | 'ollama'

// All settings from the backend
export interface Settings {
  shortcut: string
  provider: Provider
  language: string | null
  api_keys: Record<string, string>
  whisper_model_path: string | null
  remote_whisper_url: string | null
  polisher: Polisher
  ollama_url: string | null
  ollama_model: string | null
  polish_prompt: string | null
  active_preset: string | null
}

// Shortcut backend information
export interface BackendInfo {
  backend: string
  requires_restart: boolean
  compositor: string
  portal_version: number
}

// Status response from backend
export interface StatusResponse {
  state: 'Idle' | 'Recording' | 'Transcribing'
  config_valid: boolean
}

// Response when saving settings
export interface SaveSettingsResponse {
  needs_restart: boolean
}

// Whisper model info from backend
export interface WhisperModelInfo {
  name: string
  description: string
  installed: boolean
  path: string
}

// Preset info from backend
export interface PresetInfo {
  name: string
  description: string
  is_builtin: boolean
}

// Full preset details
export interface PresetDetails {
  name: string
  description: string
  prompt: string
  polisher: string | null
  model: string | null
  is_builtin: boolean
}

// Cloud provider configuration
export interface CloudProviderInfo {
  value: Provider
  label: string
  desc: string
  keyUrl: string
  placeholder: string
}

// Helper to check if provider is local
export function isLocalProvider(provider: Provider): boolean {
  return provider === 'local-whisper' || provider === 'remote-whisper'
}
