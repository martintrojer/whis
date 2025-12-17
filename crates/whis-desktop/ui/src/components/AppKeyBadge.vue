<script setup lang="ts">
defineProps<{
  placeholder?: boolean
}>()
</script>

<template>
  <span class="key-badge" :class="{ placeholder }">
    <slot />
  </span>
</template>

<style scoped>
.key-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 26px;
  padding: 0 8px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  font-family: var(--font);
  font-size: 11px;
  font-weight: 500;
  color: var(--accent);
}

.key-badge.placeholder {
  color: var(--text-weak);
}
</style>
