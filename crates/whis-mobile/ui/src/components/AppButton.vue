<script setup lang="ts">
defineProps<{
  variant?: 'primary' | 'secondary' | 'link'
  disabled?: boolean
  type?: 'button' | 'submit'
}>()
</script>

<template>
  <button
    :type="type ?? 'button'"
    class="app-button"
    :class="{
      'btn': variant !== 'link',
      'btn-primary': variant === 'primary',
      'btn-secondary': variant === 'secondary',
      'btn-link': variant === 'link',
    }"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
.app-button {
  /* Uses global .btn styles from main.css */
}
</style>
