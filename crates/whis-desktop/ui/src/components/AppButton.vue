<!-- AppButton: Button with variant styling. Props: variant (primary|secondary|link), disabled, type -->
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
      'btn-secondary': variant === 'secondary',
      'btn-link': variant === 'link',
    }"
    :disabled="disabled"
  >
    <slot />
  </button>
</template>

<style scoped>
/* Primary button (accent color) - used in ApiKeyView for download */
.app-button.btn:not(.btn-secondary):not(.btn-link) {
  background: var(--accent);
  color: var(--bg);
}

.app-button.btn:not(.btn-secondary):not(.btn-link):hover:not(:disabled) {
  filter: brightness(1.1);
}

.app-button.btn:not(.btn-secondary):not(.btn-link):focus-visible {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}
</style>
