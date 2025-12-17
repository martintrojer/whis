<script setup lang="ts">
export type TranscriptionMode = 'cloud' | 'local'

defineProps<{
  modelValue: TranscriptionMode
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TranscriptionMode]
}>()
</script>

<template>
  <div class="mode-cards">
    <button
      class="mode-card"
      :class="{ active: modelValue === 'cloud' }"
      :aria-pressed="modelValue === 'cloud'"
      @click="emit('update:modelValue', 'cloud')"
    >
      <div class="mode-card-icon" aria-hidden="true">cloud</div>
      <div class="mode-card-title">Cloud</div>
      <div class="mode-card-desc">Fast & accurate</div>
    </button>
    <button
      class="mode-card"
      :class="{ active: modelValue === 'local' }"
      :aria-pressed="modelValue === 'local'"
      @click="emit('update:modelValue', 'local')"
    >
      <div class="mode-card-icon" aria-hidden="true">device</div>
      <div class="mode-card-title">Local</div>
      <div class="mode-card-desc">Private & free</div>
    </button>
  </div>
</template>

<style scoped>
.mode-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  margin-bottom: 16px;
}

.mode-card {
  padding: 20px 16px;
  background: var(--bg-weak);
  border: 2px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  text-align: center;
  font-family: var(--font);
}

.mode-card:hover {
  border-color: var(--text-weak);
}

.mode-card:focus-visible {
  outline: none;
  border-color: var(--accent);
}

.mode-card.active {
  border-color: var(--accent);
  background: rgba(255, 213, 79, 0.08);
}

.mode-card-icon {
  font-size: 11px;
  color: var(--text-weak);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.mode-card.active .mode-card-icon {
  color: var(--accent);
}

.mode-card-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--text);
  margin-bottom: 4px;
}

.mode-card-desc {
  font-size: 11px;
  color: var(--text-weak);
}
</style>
