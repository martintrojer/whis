<script setup lang="ts">
import { ref } from 'vue'
import CommandCopy from '@/components/CommandCopy.vue'
import ImageCarousel from '@/components/ImageCarousel.vue'
import TabPanel from '@/components/TabPanel.vue'
import TheLightbox from '@/components/TheLightbox.vue'

const installTab = ref('appimage')
const lightboxOpen = ref(false)
const lightboxIndex = ref(0)

const screenshots = [
  { src: '/screenshot-00-tray.png', alt: 'Whis system tray menu showing Start Recording, Settings, and Quit options', caption: 'System tray' },
  { src: '/screenshot-01-home.png', alt: 'Whis home screen with Start Recording button', caption: 'Home' },
  { src: '/screenshot-02-shortcuts.png', alt: 'Global shortcut configuration', caption: 'Shortcuts' },
  { src: '/screenshot-03-settings-cloud.png', alt: 'Cloud provider settings with OpenAI, Groq, Deepgram options', caption: 'Cloud mode' },
  { src: '/screenshot-04-settings-local.png', alt: 'Local whisper model selection for offline transcription', caption: 'Local mode' },
  { src: '/screenshot-07-presets.png', alt: 'Preset configurations for AI prompts, email, and notes', caption: 'Presets' },
  { src: '/screenshot-08-about.png', alt: 'About page with version info', caption: 'About' },
]

function openLightbox(index: number) {
  lightboxIndex.value = index
  lightboxOpen.value = true
}
</script>

<template>
  <div class="desktop-content">
    <!-- Header -->
    <header class="view-header">
      <h1>Desktop</h1>
      <p>System tray app for voice-to-text anywhere</p>
    </header>

    <!-- Install -->
    <section id="install-desktop" class="install">
      <TabPanel
        v-model:selected="installTab"
        :tabs="[
          { value: 'appimage', label: 'AppImage' },
          { value: 'flatpak', label: 'Flatpak' },
          { value: 'deb', label: 'deb' },
          { value: 'rpm', label: 'rpm' },
          { value: 'source', label: 'source' },
          { value: 'macos', label: 'macOS' },
          { value: 'windows', label: 'Windows' },
        ]"
      >
        <div v-if="installTab === 'flatpak'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'flatpak install flathub ' },
              { text: 'ink.whis.Whis', highlight: true },
            ]"
          />
          <p class="install-note">
            Available on Flathub. Includes automatic updates.
          </p>
        </div>
        <div v-else-if="installTab === 'appimage'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'chmod +x ' },
              { text: 'Whis_*_amd64.AppImage', highlight: true },
              { text: ' && ./' },
              { text: 'Whis_*_amd64.AppImage', highlight: true },
              { text: ' --install' },
            ]"
          />
          <p class="install-note">
            Download from GitHub Releases. Then launch "Whis" from your app menu.
          </p>
        </div>
        <div v-else-if="installTab === 'deb'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'sudo apt install ' },
              { text: './Whis_*_amd64.deb', highlight: true },
            ]"
          />
          <p class="install-note">
            Download from GitHub Releases first.
          </p>
        </div>
        <div v-else-if="installTab === 'rpm'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'sudo dnf install ' },
              { text: './Whis-*.x86_64.rpm', highlight: true },
            ]"
          />
          <p class="install-note">
            Download from GitHub Releases first.
          </p>
        </div>
        <div v-else-if="installTab === 'source'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'git clone ' },
              { text: 'https://github.com/frankdierolf/whis', highlight: true },
              { text: ' && cd whis && ' },
              { text: 'just install-desktop', highlight: true },
            ]"
          />
          <p class="install-note">
            Requires <a href="https://github.com/casey/just" target="_blank" rel="noopener">just</a>.
            Builds and installs the AppImage to ~/.local/bin.
          </p>
        </div>
        <div v-else-if="installTab === 'macos'" class="panel">
          <CommandCopy
            :segments="[
              { text: 'git clone ' },
              { text: 'https://github.com/frankdierolf/whis', highlight: true },
              { text: ' && cd whis && ' },
              { text: 'just install-desktop', highlight: true },
            ]"
          />
          <p class="install-note">
            Requires <a href="https://github.com/casey/just" target="_blank" rel="noopener">just</a>.
            Builds and installs the .app to /Applications/.
          </p>
        </div>
        <div v-else class="panel">
          <CommandCopy
            :segments="[
              { text: 'git clone ' },
              { text: 'https://github.com/frankdierolf/whis', highlight: true },
              { text: ' && cd whis && ' },
              { text: 'just install-desktop', highlight: true },
            ]"
          />
          <p class="install-note">
            Experimental. Requires <a href="https://github.com/casey/just" target="_blank" rel="noopener">just</a>.
            Builds and runs the installer.
          </p>
        </div>
      </TabPanel>
    </section>

    <!-- Features -->
    <section class="features">
      <div class="section-header">
        <h2>What is Whis Desktop?</h2>
        <p>A system tray app that brings voice-to-text to your entire desktop.</p>
      </div>
      <ul>
        <li>
          <span class="marker">[*]</span>
          <div><strong>System tray</strong> Lives in your panel, always ready</div>
        </li>
        <li>
          <span class="marker">[*]</span>
          <div><strong>Global hotkey</strong> Ctrl+Alt+W from anywhere (configurable)</div>
        </li>
        <li>
          <span class="marker">[*]</span>
          <div><strong>6 providers</strong> OpenAI, Groq, Deepgram, Mistral, ElevenLabs, or local</div>
        </li>
        <li>
          <span class="marker">[*]</span>
          <div><strong>Run locally</strong> Download whisper models in-app — private and free</div>
        </li>
        <li>
          <span class="marker">[*]</span>
          <div><strong>AI post-processing</strong> Clean up transcripts with presets</div>
        </li>
        <li>
          <span class="marker">[*]</span>
          <div><strong>Cross-platform</strong> Linux, macOS, Windows (experimental)</div>
        </li>
      </ul>
    </section>

    <!-- Screenshots -->
    <section class="demo">
      <figure>
        <ImageCarousel :images="screenshots" @select="openLightbox" />
      </figure>
    </section>

    <!-- Quick Start -->
    <section class="quickstart">
      <h2>Quick Start</h2>

      <!-- AppImage -->
      <pre v-if="installTab === 'appimage'"><code><span class="comment"># 1. Download and install</span>
chmod +x <span class="highlight">Whis_*_amd64.AppImage</span>
./Whis_*_amd64.AppImage <span class="highlight">--install</span>

<span class="comment"># 2. Launch "Whis" from your app menu</span>

<span class="comment"># 3. Tray icon → Settings → Configure provider</span>

<span class="comment"># 4. Use Ctrl+Alt+W or tray menu to record!</span></code></pre>

      <!-- Flatpak -->
      <pre v-else-if="installTab === 'flatpak'"><code><span class="comment"># 1. Install from Flathub</span>
flatpak install flathub <span class="highlight">ink.whis.Whis</span>

<span class="comment"># 2. Launch "Whis" from your app menu</span>

<span class="comment"># 3. Tray icon → Settings → Configure provider</span>

<span class="comment"># 4. Use Ctrl+Alt+W or tray menu to record!</span></code></pre>

      <!-- deb -->
      <pre v-else-if="installTab === 'deb'"><code><span class="comment"># 1. Install the package</span>
sudo apt install ./<span class="highlight">Whis_*_amd64.deb</span>

<span class="comment"># 2. Launch "Whis" from your app menu</span>

<span class="comment"># 3. Tray icon → Settings → Configure provider</span>

<span class="comment"># 4. Use Ctrl+Alt+W or tray menu to record!</span></code></pre>

      <!-- rpm -->
      <pre v-else-if="installTab === 'rpm'"><code><span class="comment"># 1. Install the package</span>
sudo dnf install ./<span class="highlight">Whis-*.x86_64.rpm</span>

<span class="comment"># 2. Launch "Whis" from your app menu</span>

<span class="comment"># 3. Tray icon → Settings → Configure provider</span>

<span class="comment"># 4. Use Ctrl+Alt+W or tray menu to record!</span></code></pre>

      <!-- source -->
      <pre v-else-if="installTab === 'source'"><code><span class="comment"># 1. Install just (if not already installed)</span>
cargo install just

<span class="comment"># 2. Clone, build, and install</span>
git clone https://github.com/frankdierolf/whis && cd whis
<span class="highlight">just install-desktop</span>

<span class="comment"># 3. Launch "Whis" from your app menu</span>

<span class="comment"># 4. Tray icon → Settings → Configure provider</span></code></pre>

      <!-- macOS -->
      <pre v-else-if="installTab === 'macos'"><code><span class="comment"># 1. Install just (if not already installed)</span>
cargo install just

<span class="comment"># 2. Clone, build, and install</span>
git clone https://github.com/frankdierolf/whis && cd whis
<span class="highlight">just install-desktop</span>

<span class="comment"># 3. Launch Whis from Applications</span>

<span class="comment"># 4. Menu bar icon → Settings → Configure provider</span></code></pre>

      <!-- Windows -->
      <pre v-else><code><span class="comment"># 1. Install just (if not already installed)</span>
cargo install just

<span class="comment"># 2. Clone, build, and install</span>
git clone https://github.com/frankdierolf/whis && cd whis
<span class="highlight">just install-desktop</span>

<span class="comment"># 3. Launch Whis from Start menu</span>

<span class="comment"># 4. Tray icon → Settings → Configure provider</span>

<span class="comment"># Note: Windows support is experimental</span></code></pre>
    </section>

    <!-- Lightbox -->
    <TheLightbox
      v-model:open="lightboxOpen"
      :images="screenshots"
      :initial-index="lightboxIndex"
    />
  </div>
</template>

<style scoped>
.desktop-content {
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

.install {
  padding: var(--vertical-padding) var(--padding);
}

.panel {
  display: block;
}

.install-cmd {
  margin-top: 0.75rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg);
  border: 1px solid var(--border-weak);
  border-radius: 4px;
  font-size: 0.8rem;
  overflow-x: auto;
}

.install-cmd code {
  display: block;
  white-space: nowrap;
}

.install-note {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  color: var(--text-weak);
}

.features {
  border-top: 1px solid var(--border-weak);
  padding: var(--vertical-padding) var(--padding);
}

.section-header {
  margin-bottom: 2rem;
}

.section-header h2 {
  font-size: 1.1rem;
  font-weight: 500;
  color: var(--text-strong);
  margin-bottom: 0.5rem;
}

.section-header p {
  color: var(--text);
}

.features ul {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.features li {
  display: flex;
  gap: 0.75rem;
  line-height: 1.6;
}

.marker {
  color: var(--icon);
  flex-shrink: 0;
}

.features li strong {
  color: var(--text-strong);
  font-weight: 500;
  margin-right: 0.5rem;
}

.demo {
  border-top: 1px solid var(--border-weak);
  padding: var(--vertical-padding) var(--padding);
}

.demo figure {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.quickstart {
  border-top: 1px solid var(--border-weak);
  padding: var(--vertical-padding) var(--padding);
}

.quickstart h2 {
  font-size: 1.1rem;
  font-weight: 500;
  color: var(--text-strong);
  margin-bottom: 1.5rem;
}

.quickstart pre {
  background: var(--bg-weak);
  border: 1px solid var(--border-weak);
  border-radius: 6px;
  padding: 1.5rem;
  overflow-x: auto;
}

.quickstart code {
  font-family: var(--font);
  font-size: 0.9rem;
  line-height: 1.8;
}

.comment {
  color: var(--text-weak);
}

.highlight {
  color: var(--text-strong);
  font-weight: 500;
}
</style>
