<!-- AppSlider: Range slider with value display. Props: modelValue, min, max, step, unit, disabled -->
<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  modelValue: number
  min: number
  max: number
  step: number
  unit?: string
  disabled?: boolean
  ariaLabel?: string
}>(), {
  unit: '',
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const displayValue = computed(() => {
  return props.unit ? `${props.modelValue} ${props.unit}` : String(props.modelValue)
})

function handleInput(event: Event) {
  const value = Number((event.target as HTMLInputElement).value)
  emit('update:modelValue', value)
}
</script>

<template>
  <div class="slider-container" :class="{ disabled }">
    <input
      type="range"
      class="slider"
      :value="modelValue"
      :min="min"
      :max="max"
      :step="step"
      :disabled="disabled"
      :aria-label="ariaLabel"
      @input="handleInput"
    >
    <span class="slider-value">{{ displayValue }}</span>
  </div>
</template>

<style scoped>
.slider-container {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
}

.slider {
  flex: 1;
  height: 4px;
  background: var(--bg-weak);
  border-radius: 2px;
  outline: none;
  -webkit-appearance: none;
  appearance: none;
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  background: var(--accent);
  border-radius: 50%;
  cursor: pointer;
  transition: background 0.15s ease;
}

.slider::-webkit-slider-thumb:hover {
  background: var(--text);
}

.slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  background: var(--accent);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: background 0.15s ease;
}

.slider::-moz-range-thumb:hover {
  background: var(--text);
}

.slider:focus {
  outline: none;
}

.slider:focus::-webkit-slider-thumb {
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.1);
}

.slider:focus::-moz-range-thumb {
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.1);
}

.slider-value {
  font-size: 12px;
  color: var(--text);
  min-width: 80px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.slider-container.disabled .slider {
  opacity: 0.5;
  cursor: not-allowed;
}

.slider-container.disabled .slider-value {
  opacity: 0.5;
}
</style>
