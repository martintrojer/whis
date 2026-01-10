<!-- Accordion: Collapsible content container. Only one item expanded at a time -->
<script setup lang="ts">
import { provide, ref } from 'vue'

// Track which item is expanded (only one at a time)
const expandedItem = ref<string | null>(null)

provide('accordion', {
  expandedItem,
  toggle(id: string) {
    expandedItem.value = expandedItem.value === id ? null : id
  },
})
</script>

<template>
  <div class="accordion">
    <slot />
  </div>
</template>

<style scoped>
.accordion {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-weak);
  padding: 0 16px;
}
</style>
