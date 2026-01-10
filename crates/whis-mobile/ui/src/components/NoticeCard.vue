<!-- NoticeCard: Info or warning notice box. Props: marker ([!],[*],[i]), variant (default|warning) -->
<script setup lang="ts">
defineProps<{
  marker?: string // [!], [*], [i]
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
/* Uses global .notice styles from main.css */
/* Component-specific overrides if needed */
.notice-content :deep(p) {
  margin: 0;
}

.notice-content :deep(strong) {
  color: var(--text-strong);
}
</style>
