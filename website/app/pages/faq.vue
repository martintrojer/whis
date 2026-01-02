<script setup lang="ts">
useHead({
  title: 'FAQ - whis',
  link: [
    { rel: 'canonical', href: 'https://whis.ink/faq' },
  ],
  meta: [
    { name: 'description', content: 'Frequently asked questions about whis voice-to-text tool.' },
    { property: 'og:title', content: 'FAQ - whis' },
    { property: 'og:description', content: 'Frequently asked questions about whis voice-to-text tool.' },
    { property: 'og:url', content: 'https://whis.ink/faq' },
    { property: 'og:image', content: 'https://whis.ink/og-image.jpg' },
    { property: 'og:type', content: 'website' },
    { name: 'twitter:card', content: 'summary_large_image' },
    { name: 'twitter:title', content: 'FAQ - whis' },
    { name: 'twitter:description', content: 'Frequently asked questions about whis voice-to-text tool.' },
    { name: 'twitter:image', content: 'https://whis.ink/og-image.jpg' },
  ],
})

const faqItems = [
  {
    question: 'What is post-processing?',
    answer:
      'AI-powered cleanup of your transcript. It removes filler words (um, uh, like) and fixes grammar while keeping technical terms. Use <code>--post-process</code> flag or enable in Desktop settings.',
  },
  {
    question: 'What do I need?',
    answer:
      '<strong>Cloud mode:</strong> An API key from your chosen provider.<br><strong>Local mode:</strong> Just run <code>whis setup local</code> (downloads ~500MB model).<br><strong>CLI only:</strong> Requires FFmpeg for audio encoding.',
  },
  {
    question: 'What about my data?',
    answer:
      '<strong>Cloud:</strong> Audio is sent to your provider\'s servers for processing.<br><strong>Local:</strong> Everything stays on your machine—never leaves.<br>whis doesn\'t store recordings or transcripts.',
  },
  {
    question: 'What are presets?',
    answer:
      'Output formats for different use cases. Built-in: <code>ai-prompt</code>, <code>email</code>, <code>default</code>. Use with <code>whis --as email</code> or create your own with <code>whis presets new</code>.',
  },
  {
    question: 'Where is my config stored?',
    answer:
      'Settings: <code>~/.config/whis/settings.json</code><br>Presets: <code>~/.config/whis/presets/</code><br>Models: <code>~/.local/share/whis/models/</code><br>View current config: <code>whis config --show</code>',
  },
  {
    question: 'Can I pipe output to other tools?',
    answer:
      'Yes! Use <code>--print</code> to output to stdout instead of clipboard:<br><code>whis --print | pbcopy</code><br><code>whis -f audio.mp3 --print > transcript.txt</code>',
  },
  {
    question: 'How do I uninstall?',
    answer:
      '<strong>Flatpak:</strong> <code>flatpak uninstall ink.whis.Whis</code><br><strong>AppImage:</strong> Run with <code>--uninstall</code> flag, then delete the file.<br><strong>Other:</strong> Remove the binary and <code>~/.config/whis/</code> folder.',
  },
]
</script>

<template>
  <div class="faq-content">
    <ViewHeader title="FAQ" subtitle="Common questions about whis" />

    <section class="faq">
      <FaqAccordion :items="faqItems" />
    </section>
  </div>
</template>

<style scoped>
.faq-content {
  padding: 2rem;
}

.faq {
  padding: var(--vertical-padding) var(--padding);
}
</style>
