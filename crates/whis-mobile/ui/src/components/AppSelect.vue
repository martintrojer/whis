<script setup lang="ts">
import type { SelectOption } from '../types'
import { computed, onMounted, onUnmounted, ref } from 'vue'

const props = defineProps<{
  modelValue: string | null
  options: SelectOption[]
  disabled?: boolean
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | null]
}>()

const isOpen = ref(false)
const selectRef = ref<HTMLElement | null>(null)

const selectedLabel = computed(() => {
  const opt = props.options.find(o => o.value === props.modelValue)
  return opt?.label || ''
})

function toggle() {
  if (!props.disabled)
    isOpen.value = !isOpen.value
}

function select(value: string | null) {
  emit('update:modelValue', value)
  isOpen.value = false
}

function handleClickOutside(e: MouseEvent) {
  if (selectRef.value && !selectRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div
    ref="selectRef"
    class="select"
    :class="{ open: isOpen, disabled }"
  >
    <button
      type="button"
      class="select-trigger"
      :disabled="disabled"
      :aria-label="ariaLabel"
      :aria-expanded="isOpen"
      @click="toggle"
    >
      <span class="select-value">{{ selectedLabel }}</span>
      <svg class="select-chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M6 9l6 6 6-6" />
      </svg>
    </button>
    <div v-show="isOpen" class="select-dropdown" role="listbox">
      <button
        v-for="opt in options"
        :key="opt.value ?? 'null'"
        type="button"
        class="select-option"
        :class="{ selected: opt.value === modelValue }"
        :disabled="opt.disabled"
        role="option"
        :aria-selected="opt.value === modelValue"
        @click="select(opt.value)"
      >
        {{ opt.label }}
      </button>
    </div>
  </div>
</template>

<style scoped>
/* Uses global .select styles from main.css */
/* Component-specific overrides if needed */
.select-value {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
