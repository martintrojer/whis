<script setup lang="ts">
defineProps<{
  question: string
}>()

const expanded = defineModel<boolean>('expanded', { default: false })

function toggle() {
  expanded.value = !expanded.value
}
</script>

<template>
  <li class="faq-item" :data-expanded="expanded || undefined" :data-closed="!expanded || undefined">
    <button class="faq-question" @click="toggle">
      <span class="icon-plus">+</span>
      <span class="icon-minus">&minus;</span>
      <span>{{ question }}</span>
    </button>
    <div class="faq-answer">
      <slot />
    </div>
  </li>
</template>

<style scoped>
.faq-item {
  border-bottom: 1px solid var(--border-weak);
}

.faq-item:last-child {
  border-bottom: none;
}

.faq-question {
  all: unset;
  display: flex;
  align-items: flex-start;
  gap: 1rem;
  width: 100%;
  padding: 1rem 0;
  cursor: pointer;
  color: var(--text-strong);
  font-weight: 500;
  transition: color 0.15s ease;
}

.faq-question:hover {
  color: var(--text-strong);
}

.faq-question:hover .icon-plus,
.faq-question:hover .icon-minus {
  color: var(--accent);
}

.icon-plus,
.icon-minus {
  flex-shrink: 0;
  width: 1rem;
  text-align: center;
  color: var(--icon);
  font-weight: 400;
}

/* Closed state */
.faq-item[data-closed] .icon-plus {
  display: inline;
}

.faq-item[data-closed] .icon-minus {
  display: none;
}

.faq-item[data-closed] .faq-answer {
  display: none;
}

/* Expanded state */
.faq-item[data-expanded] .icon-plus {
  display: none;
}

.faq-item[data-expanded] .icon-minus {
  display: inline;
}

.faq-item[data-expanded] .faq-answer {
  display: block;
}

.faq-answer {
  padding: 0 0 1rem 2rem;
  color: var(--text);
}

.faq-answer :deep(code) {
  background: var(--bg-weak);
  padding: 0.15em 0.4em;
  border-radius: 3px;
  font-size: 0.9em;
}
</style>
