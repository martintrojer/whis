<!-- AppInput: Text input with v-model. Props: modelValue, type (text|password), placeholder, disabled -->
<script setup lang="ts">
defineProps<{
  modelValue: string
  type?: 'text' | 'password'
  placeholder?: string
  disabled?: boolean
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

function handleInput(event: Event) {
  const value = (event.target as HTMLInputElement).value
  emit('update:modelValue', value)
}
</script>

<template>
  <input
    class="app-input"
    :type="type ?? 'text'"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    :aria-label="ariaLabel"
    spellcheck="false"
    autocomplete="off"
    @input="handleInput"
  >
</template>

<style scoped>
.app-input {
  padding: 10px 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 12px;
  color: var(--text);
  transition: border-color 0.15s ease;
  min-width: 0;
  width: 100%;
}

.app-input:focus {
  outline: none;
  border-color: var(--accent);
}

.app-input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.app-input::placeholder {
  color: var(--text-weak);
}
</style>
