<script setup lang="ts">
import FaqAccordion from '@/components/FaqAccordion.vue'

const faqItems = [
  {
    question: 'CLI or Desktop — which should I use?',
    answer:
      '<strong>CLI</strong> if you live in the terminal and want voice-to-text in your shell workflow. <strong>Desktop</strong> if you want a system-wide hotkey that works from any app.',
  },
  {
    question: 'What providers are supported?',
    answer:
      'Six providers: <strong>OpenAI</strong>, <strong>Groq</strong> (fastest, cheapest), <strong>Deepgram</strong>, <strong>Mistral</strong>, <strong>ElevenLabs</strong>, and <strong>Local Whisper</strong> (fully offline via whisper.cpp).',
  },
  {
    question: 'Can I use it offline?',
    answer:
      'Yes! Run <code>whis setup local</code> to download a whisper model and use Ollama for post-processing. Zero cloud dependency, completely private.',
  },
  {
    question: 'What is post-processing?',
    answer:
      'AI-powered cleanup of your transcript. Removes filler words (um, uh, like), fixes grammar, and preserves technical terms. Use <code>--post-process</code> or enable it in settings.',
  },
  {
    question: 'What do I need?',
    answer:
      'For cloud: an API key from your chosen provider. For local: just run <code>whis setup local</code> to download models. CLI also needs FFmpeg.',
  },
  {
    question: 'How does hotkey mode work?',
    answer:
      'Run <code>whis listen</code> to start the background service. Hit Ctrl+Alt+W to toggle recording from anywhere. Customize with <code>--hotkey</code>.',
  },
  {
    question: 'What about my data?',
    answer:
      'With cloud providers, audio goes to their API. With local mode, everything stays on your machine. whis doesn\'t store anything.',
  },
  {
    question: 'macOS? Windows?',
    answer: 'All three platforms supported! Linux (X11 + Wayland), macOS (tested), and Windows (experimental).',
  },
  {
    question: 'How do I uninstall?',
    answer:
      'Flatpak: <code>flatpak uninstall ink.whis.Whis</code>. AppImage: <code>./Whis_*_amd64.AppImage --uninstall</code> then delete the file.',
  },
]
</script>

<template>
  <div class="faq-content">
    <!-- Header -->
    <header class="view-header">
      <h1>FAQ</h1>
      <p>Common questions about whis</p>
    </header>

    <section class="faq">
      <FaqAccordion :items="faqItems" />
    </section>
  </div>
</template>

<style scoped>
.faq-content {
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

.faq {
  padding: var(--vertical-padding) var(--padding);
}
</style>
