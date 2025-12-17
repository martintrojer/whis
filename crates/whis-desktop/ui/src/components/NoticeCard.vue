<script setup lang="ts">
defineProps<{
  marker?: string  // [!], [*], [i]
  variant?: 'default' | 'warning'
}>()
</script>

<template>
  <div class="notice" :class="{ warning: variant === 'warning' }">
    <span class="notice-marker" aria-hidden="true">{{ marker ?? '[*]' }}</span>
    <div class="notice-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.notice {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
}

.notice-marker {
  color: var(--accent);
  flex-shrink: 0;
}

.notice-content {
  font-size: 12px;
  color: var(--text);
  line-height: 1.5;
  min-width: 0;
  word-wrap: break-word;
}

.notice-content :deep(p) {
  margin: 0;
}

.notice-content :deep(strong) {
  color: var(--text-strong);
}

.notice.warning {
  border-color: var(--warning, #f59e0b);
}

.notice.warning .notice-marker {
  color: var(--warning, #f59e0b);
}
</style>
