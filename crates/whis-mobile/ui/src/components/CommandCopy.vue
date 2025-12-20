<script setup lang="ts">
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { computed, ref } from 'vue'
import IconCheck from './icons/IconCheck.vue'
import IconCopy from './icons/IconCopy.vue'

interface Segment {
  text: string
  highlight?: boolean
}

const props = defineProps<{
  segments: Segment[]
}>()

const copied = ref(false)

const fullText = computed(() => props.segments.map(s => s.text).join(''))

async function copy() {
  try {
    await writeText(fullText.value)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 1500)
  }
  catch (e) {
    console.error('Failed to copy:', e)
  }
}
</script>

<template>
  <button class="command" @click="copy">
    <code>
      <span
        v-for="(segment, i) in props.segments"
        :key="i"
        :class="segment.highlight ? 'highlight' : 'dim'"
      >{{ segment.text }}</span>
    </code>
    <span class="copy-status">
      <IconCheck v-if="copied" class="check" />
      <IconCopy v-else class="copy" />
    </span>
  </button>
</template>

<style scoped>
.command {
  all: unset;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 12px 16px;
  min-height: var(--touch-target-min);
  border-radius: var(--radius);
  cursor: pointer;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  box-sizing: border-box;
  transition: background 0.15s ease;
}

.command:active {
  background: var(--bg-hover);
}

.command:active .copy-status svg {
  color: var(--text-strong);
}

.command code {
  font-family: var(--font);
  font-size: 14px;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dim {
  color: var(--text-weak);
}

.highlight {
  color: var(--text-strong);
  font-weight: 500;
}

.copy-status {
  flex-shrink: 0;
  display: flex;
  align-items: center;
}

.copy-status svg {
  color: var(--icon);
}

.copy-status .check {
  color: var(--accent);
}
</style>
