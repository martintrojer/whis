<script setup lang="ts">
import { computed, ref } from 'vue'

const props = defineProps<{
  modelValue: string
  type?: 'text' | 'password'
  placeholder?: string
  disabled?: boolean
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const showPassword = ref(false)

const inputType = computed(() => {
  if (props.type === 'password') {
    return showPassword.value ? 'text' : 'password'
  }
  return props.type ?? 'text'
})

function handleInput(event: Event) {
  const value = (event.target as HTMLInputElement).value
  emit('update:modelValue', value)
}

function togglePassword() {
  showPassword.value = !showPassword.value
}
</script>

<template>
  <div class="input-wrapper">
    <input
      class="input"
      :type="inputType"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :aria-label="ariaLabel"
      spellcheck="false"
      autocomplete="off"
      @input="handleInput"
    >
    <button
      v-if="type === 'password'"
      type="button"
      class="toggle-btn"
      :aria-label="showPassword ? 'Hide password' : 'Show password'"
      @click="togglePassword"
    >
      <svg v-if="showPassword" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24" />
        <line x1="1" y1="1" x2="23" y2="23" />
      </svg>
      <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.input-wrapper {
  position: relative;
  width: 100%;
}

.input {
  /* Uses global .input styles from main.css */
  padding-right: 48px;
}

.input-wrapper:not(:has(.toggle-btn)) .input {
  padding-right: 16px;
}

.toggle-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: none;
  border: none;
  color: var(--icon);
  cursor: pointer;
  padding: 0;
}

.toggle-btn:active {
  color: var(--text);
}

.toggle-btn svg {
  width: 20px;
  height: 20px;
}
</style>
