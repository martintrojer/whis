<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { computed, onMounted, onUnmounted, ref } from 'vue'

type BubbleState = 'idle' | 'recording' | 'transcribing'

const state = ref<BubbleState>('idle')
const isVisible = ref(false)

// Map state to SVG icon
const iconSrc = computed(() => {
  switch (state.value) {
    case 'recording':
      return '/icons/bubble-recording.svg'
    case 'transcribing':
      return '/icons/bubble-processing.svg'
    default:
      return '/icons/bubble-idle.svg'
  }
})

let unlistenState: (() => void) | null = null
let unlistenHide: (() => void) | null = null

onMounted(async () => {
  // Listen for state changes from Rust
  unlistenState = await listen<BubbleState>('bubble-state', (event) => {
    state.value = event.payload
    isVisible.value = true
  })

  // Listen for hide signal
  unlistenHide = await listen('bubble-hide', () => {
    isVisible.value = false
  })
})

onUnmounted(() => {
  unlistenState?.()
  unlistenHide?.()
})

function handleClick() {
  invoke('bubble_toggle_recording')
}
</script>

<template>
  <div
    class="bubble"
    :class="{ visible: isVisible }"
    @click="handleClick"
  >
    <img :src="iconSrc" alt="Whis" class="icon">
  </div>
</template>

<style scoped>
.bubble {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 200ms ease, transform 100ms ease;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

.bubble.visible {
  opacity: 1;
}

.bubble:hover {
  transform: scale(1.05);
}

.bubble:active {
  transform: scale(0.95);
}

.icon {
  width: 32px;
  height: 32px;
}
</style>
