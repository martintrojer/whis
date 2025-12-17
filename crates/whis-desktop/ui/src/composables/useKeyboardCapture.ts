import { ref, computed } from 'vue'

/**
 * Composable for capturing keyboard shortcuts.
 * Used in ShortcutView for recording global shortcuts.
 */
export function useKeyboardCapture(initialValue: string = '') {
  const isRecording = ref(false)
  const capturedShortcut = ref(initialValue)

  // Split shortcut into individual keys for display
  const shortcutKeys = computed(() => {
    if (capturedShortcut.value === 'Press keys...') {
      return ['...']
    }
    return capturedShortcut.value.split('+')
  })

  function handleKeyDown(e: KeyboardEvent) {
    if (!isRecording.value) return
    e.preventDefault()

    const keys: string[] = []
    if (e.ctrlKey) keys.push('Ctrl')
    if (e.shiftKey) keys.push('Shift')
    if (e.altKey) keys.push('Alt')
    if (e.metaKey) keys.push('Super')

    const key = e.key.toUpperCase()
    if (!['CONTROL', 'SHIFT', 'ALT', 'META'].includes(key)) {
      keys.push(key)
    }

    if (keys.length > 0) {
      capturedShortcut.value = keys.join('+')
    }
  }

  function startRecording() {
    isRecording.value = true
    capturedShortcut.value = 'Press keys...'
  }

  function stopRecording() {
    isRecording.value = false
  }

  function setShortcut(value: string) {
    capturedShortcut.value = value
  }

  return {
    isRecording,
    capturedShortcut,
    shortcutKeys,
    handleKeyDown,
    startRecording,
    stopRecording,
    setShortcut,
  }
}
