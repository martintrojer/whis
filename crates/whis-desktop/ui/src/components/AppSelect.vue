<script setup lang="ts">
import type { SelectOption } from '../types'

defineProps<{
  modelValue: string | null
  options: SelectOption[]
  disabled?: boolean
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string | null]
}>()

function handleChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  emit('update:modelValue', value === '' ? null : value)
}
</script>

<template>
  <select
    class="app-select"
    :value="modelValue ?? ''"
    :disabled="disabled"
    :aria-label="ariaLabel"
    @change="handleChange"
  >
    <option
      v-for="opt in options"
      :key="opt.value ?? 'null'"
      :value="opt.value ?? ''"
    >
      {{ opt.label }}
    </option>
  </select>
</template>

<style scoped>
.app-select {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  cursor: pointer;
  transition: border-color 0.15s ease;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23808080' d='M3 4.5L6 7.5L9 4.5'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}

.app-select:focus {
  outline: none;
  border-color: var(--accent);
}

.app-select:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.app-select option {
  background: var(--bg);
  color: var(--text);
}
</style>
