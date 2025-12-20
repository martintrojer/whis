<script setup lang="ts">
</script>

<template>
  <div class="mobile-content">
    <!-- Header -->
    <header class="view-header">
      <h1>Mobile</h1>
      <p>Voice-to-text on the go</p>
    </header>

    <!-- Teaser -->
    <section class="teaser">
      <div class="status-badge">
        [experimental]
      </div>

      <p class="teaser-text">
        Mobile app in early development. Same whis experience, from your phone.
      </p>

      <ul class="features">
        <li>
          <span class="marker">[*]</span>
          <span>Voice-to-text from your phone</span>
        </li>
        <li>
          <span class="marker">[*]</span>
          <span>Same providers as CLI & Desktop</span>
        </li>
        <li>
          <span class="marker">[*]</span>
          <span>Text straight to clipboard</span>
        </li>
      </ul>

      <div class="cta">
        <a
          href="https://github.com/frankdierolf/whis"
          target="_blank"
          rel="noopener"
          class="watch-link"
        >
          Watch on GitHub
        </a>
      </div>
    </section>
  </div>
</template>

<style scoped>
.mobile-content {
  padding: 2rem;
}

.view-header {
  margin-bottom: 2rem;
  padding-bottom: 1.5rem;
  border-bottom: 1px solid var(--border-weak);
}

.view-header h1 {
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--text-strong);
  margin-bottom: 0.5rem;
}

.view-header p {
  font-size: 0.9rem;
  color: var(--text-weak);
}

.teaser {
  padding: var(--vertical-padding) var(--padding);
}

.status-badge {
  display: inline-block;
  font-size: 0.75rem;
  color: var(--accent);
  margin-bottom: 1rem;
}

.teaser-text {
  font-size: 0.95rem;
  color: var(--text);
  margin-bottom: 1.5rem;
  max-width: 28rem;
  line-height: 1.6;
}

.features {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-bottom: 2rem;
}

.features li {
  display: flex;
  gap: 0.75rem;
  font-size: 0.9rem;
  color: var(--text);
}

.marker {
  color: var(--icon);
  flex-shrink: 0;
}

.cta {
  padding-top: 1rem;
  border-top: 1px solid var(--border-weak);
}

.watch-link {
  font-size: 0.85rem;
  color: var(--text-strong);
  text-decoration: underline;
  text-underline-offset: 2px;
  text-decoration-thickness: 1px;
  transition: color 0.15s ease;
}

.watch-link:hover {
  color: var(--accent);
}
</style>
