<!-- AccordionItem: Expandable section within Accordion. Props: title, v-model:expanded -->
<script setup lang="ts">
defineProps<{
  title: string
}>()

const expanded = defineModel<boolean>('expanded', { default: false })

function toggle() {
  expanded.value = !expanded.value
}
</script>

<template>
  <div class="accordion-item" :data-expanded="expanded || undefined" :data-closed="!expanded || undefined">
    <button class="accordion-header" @click="toggle">
      <span class="icon-plus">+</span>
      <span class="icon-minus">&minus;</span>
      <span class="accordion-title">{{ title }}</span>
    </button>
    <div class="accordion-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.accordion-item {
  border-bottom: 1px solid var(--border-weak);
}

.accordion-item:last-child {
  border-bottom: none;
}

.accordion-header {
  all: unset;
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 16px 0;
  min-height: var(--touch-target-min);
  cursor: pointer;
  box-sizing: border-box;
}

.accordion-header:active {
  background: var(--bg-hover);
  margin: 0 -16px;
  padding: 16px;
  width: calc(100% + 32px);
}

.accordion-title {
  color: var(--text-strong);
  font-weight: 500;
  font-size: 14px;
}

.icon-plus,
.icon-minus {
  flex-shrink: 0;
  width: 1rem;
  text-align: center;
  color: var(--icon);
  font-weight: 400;
  transition: color 0.15s ease;
}

.accordion-header:active .icon-plus,
.accordion-header:active .icon-minus {
  color: var(--accent);
}

/* Closed state */
.accordion-item[data-closed] .icon-plus {
  display: inline;
}

.accordion-item[data-closed] .icon-minus {
  display: none;
}

.accordion-item[data-closed] .accordion-content {
  display: none;
}

/* Expanded state */
.accordion-item[data-expanded] .icon-plus {
  display: none;
}

.accordion-item[data-expanded] .icon-minus {
  display: inline;
  color: var(--accent);
}

.accordion-item[data-expanded] .accordion-content {
  display: block;
}

.accordion-content {
  padding: 0 0 16px 28px;
  color: var(--text);
  font-size: 14px;
  line-height: 1.5;
}

.accordion-content :deep(code) {
  background: var(--bg-weak);
  padding: 0.15em 0.4em;
  border-radius: 3px;
  font-size: 0.9em;
}
</style>
