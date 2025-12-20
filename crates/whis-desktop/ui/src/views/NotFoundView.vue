<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

function goHome() {
  router.push({ name: 'home' })
}
</script>

<template>
  <section class="section">
    <header class="section-header">
      <h1>Not Found</h1>
      <p>Page doesn't exist</p>
    </header>

    <div class="section-content">
      <div class="not-found-icon" aria-hidden="true">
        [?]
      </div>

      <p class="not-found-message">
        The page you're looking for couldn't be found.
      </p>

      <button class="btn btn-secondary" @click="goHome">
        Go to Home
      </button>
    </div>
  </section>
</template>

<style scoped>
.not-found-icon {
  font-size: 2rem;
  color: var(--text-weak);
  margin-bottom: 16px;
}

.not-found-message {
  font-size: 13px;
  color: var(--text);
  margin-bottom: 20px;
}
</style>
