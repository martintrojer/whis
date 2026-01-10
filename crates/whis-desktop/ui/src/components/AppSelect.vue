<!-- AppSelect: Custom dropdown select. Props: modelValue, options (SelectOption[]), disabled -->
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
    class="custom-select"
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
      <span class="select-chevron">›</span>
    </button>
    <div v-show="isOpen" class="select-dropdown" role="listbox">
      <button
        v-for="opt in options"
        :key="opt.value ?? 'null'"
        type="button"
        class="select-option"
        :class="{ selected: opt.value === modelValue }"
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
.custom-select {
  position: relative;
}

.select-trigger {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  cursor: pointer;
  transition: border-color 0.15s ease;
}

.select-trigger:hover:not(:disabled) {
  border-color: var(--text-weak);
}

.select-trigger:focus {
  outline: none;
  border-color: var(--accent);
}

.custom-select.open .select-trigger {
  border-color: var(--accent);
}

.select-value {
  flex: 1;
  text-align: left;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.select-chevron {
  transform: rotate(90deg);
  transition: transform 0.15s ease;
  color: var(--text-weak);
  font-size: 14px;
  margin-left: 8px;
}

.custom-select.open .select-chevron {
  transform: rotate(-90deg);
}

.select-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 100;
  max-height: 200px;
  overflow-y: auto;
}

.select-option {
  width: 100%;
  padding: 8px 10px;
  background: none;
  border: none;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  cursor: pointer;
  text-align: left;
  transition: background 0.1s ease;
}

.select-option:hover {
  background: var(--bg-weak);
}

.select-option.selected {
  color: var(--accent);
}

.custom-select.disabled .select-trigger {
  opacity: 0.6;
  cursor: not-allowed;
}
</style>
