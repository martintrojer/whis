<script setup lang="ts">
import { ref } from 'vue'
import IconChevronLeft from './icons/IconChevronLeft.vue'
import IconChevronRight from './icons/IconChevronRight.vue'

defineProps<{
  images: { src: string, alt: string, caption?: string }[]
}>()

const emit = defineEmits<{
  select: [index: number]
}>()

const carousel = ref<HTMLElement | null>(null)

function scrollPrev() {
  if (carousel.value) {
    carousel.value.scrollBy({ left: -carousel.value.offsetWidth, behavior: 'smooth' })
  }
}

function scrollNext() {
  if (carousel.value) {
    carousel.value.scrollBy({ left: carousel.value.offsetWidth, behavior: 'smooth' })
  }
}
</script>

<template>
  <div class="carousel-wrapper">
    <button class="carousel-btn prev" aria-label="Previous screenshot" @click="scrollPrev">
      <IconChevronLeft />
    </button>
    <div ref="carousel" class="carousel">
      <figure v-for="(image, index) in images" :key="index" @click="emit('select', index)">
        <img :src="image.src" :alt="image.alt" loading="lazy">
        <figcaption v-if="image.caption">
          {{ image.caption }}
        </figcaption>
      </figure>
    </div>
    <button class="carousel-btn next" aria-label="Next screenshot" @click="scrollNext">
      <IconChevronRight />
    </button>
  </div>
</template>

<style scoped>
.carousel-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.carousel {
  flex: 1;
  display: flex;
  gap: 1rem;
  overflow-x: auto;
  scroll-snap-type: x mandatory;
  scroll-behavior: smooth;
  -webkit-overflow-scrolling: touch;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.carousel::-webkit-scrollbar {
  display: none;
}

.carousel figure {
  flex-shrink: 0;
  width: 100%;
  scroll-snap-align: start;
  margin: 0;
  text-align: center;
  cursor: pointer;
}

.carousel img {
  width: 100%;
  height: auto;
  border-radius: 8px;
  border: 1px solid var(--border-weak);
}

.carousel figcaption {
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-weak);
}

.carousel-btn {
  appearance: none;
  background: var(--bg-weak);
  border: 1px solid var(--border-weak);
  color: var(--text);
  width: 2.5rem;
  height: 2.5rem;
  border-radius: 50%;
  cursor: pointer;
  font-size: 1rem;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    background 0.15s ease,
    color 0.15s ease;
}

.carousel-btn:hover {
  background: var(--bg-hover);
  color: var(--text-strong);
}
</style>
