<!-- TabPanel: Tab navigation with content slots. Props: tabs, v-model:selected -->
<script setup lang="ts">
defineProps<{
  tabs: { value: string, label: string }[]
}>()

const selected = defineModel<string>('selected', { required: true })
</script>

<template>
  <div class="tabs">
    <div class="tablist" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab.value"
        role="tab"
        class="tab"
        :aria-selected="selected === tab.value"
        @click="selected = tab.value"
      >
        {{ tab.label }}
      </button>
    </div>
    <div class="panels">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.tabs {
  width: 100%;
}

.tablist {
  display: flex;
  gap: 1.5rem;
  border-bottom: 1px solid var(--border-weak);
  margin-bottom: 0;
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}

.tab {
  appearance: none;
  background: transparent;
  border: none;
  padding: 14px 4px;
  min-height: var(--touch-target-min);
  font-family: var(--font);
  font-size: 14px;
  color: var(--text-weak);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  white-space: nowrap;
  transition:
    color 0.15s ease,
    border-color 0.15s ease;
}

.tab:active {
  color: var(--text-strong);
}

.tab[aria-selected='true'] {
  color: var(--accent);
  border-bottom-color: var(--accent);
  font-weight: 500;
}

.panels {
  background: var(--bg-weak);
  border: 1px solid var(--border-weak);
  border-top: none;
  border-radius: 0 0 var(--radius) var(--radius);
  padding: 16px;
}
</style>
