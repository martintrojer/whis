<script setup lang="ts">
import { ref } from 'vue'
import FaqItem from './FaqItem.vue'

defineProps<{
  items: { question: string, answer: string }[]
}>()

const expandedIndex = ref<number | null>(null)

function isExpanded(index: number): boolean {
  return expandedIndex.value === index
}

function setExpanded(index: number, value: boolean) {
  expandedIndex.value = value ? index : null
}
</script>

<template>
  <ul class="faq-list">
    <FaqItem
      v-for="(item, index) in items"
      :key="index"
      :question="item.question"
      :expanded="isExpanded(index)"
      @update:expanded="setExpanded(index, $event)"
    >
      <p v-html="item.answer" />
    </FaqItem>
  </ul>
</template>

<style scoped>
.faq-list {
  display: flex;
  flex-direction: column;
}
</style>
