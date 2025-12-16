# Chapter 23: Vue Frontend & Tauri Integration

The Whis desktop UI is built with Vue 3.6 alpha, specifically using **Vapor Mode**—Vue's upcoming compile-time optimized rendering strategy. This is a deliberate choice: Whis is a personal hobby project, and Frank is a Vue fan eagerly waiting for Vapor's stable release.

In this chapter, we'll explore:
- Why Vapor Mode, and how it differs from traditional Vue
- The component structure (App.vue, HomeView, ShortcutView, ApiKeyView)
- How Vue components call Rust commands via `@tauri-apps/api`
- Reactive state management without Pinia/Vuex
- The build process with Vite and rolldown

## Why Vue 3.6 Vapor Mode?

Traditional Vue uses a **Virtual DOM**: the framework builds a JavaScript representation of the DOM tree, diffs it on updates, and patches the real DOM. This works well but has overhead—every component update requires diffing, even if only one reactive value changed.

**Vapor Mode** is Vue's answer to Solid.js and Svelte: **no Virtual DOM**. The compiler analyzes your template at build time and generates granular reactivity code that directly updates the DOM when specific reactive values change. Think of it as Vue's version of "compiled away"—you write Vue, but the output is optimized imperative DOM updates.

Benefits for Whis:
- **Smaller bundle**: No vdom runtime (Whis UI bundle is ~40KB gzipped)
- **Faster updates**: Direct DOM manipulation, no diffing
- **Still Vue**: Composition API, `ref()`, `computed()`, all work as expected

The tradeoff: Vapor Mode is **alpha** (as of 3.6.0-alpha.5). Production apps shouldn't use it yet. But for a personal desktop app with controlled deployment, it's perfect for experimenting with the future of Vue.

From `package.json` (line 13):

```json
"dependencies": {
  "@tauri-apps/api": "^2.0.0",
  "@tauri-apps/plugin-process": "^2.3.1",
  "vue": "^3.6.0-alpha.5"
}
```

**From `whis-desktop/ui/package.json:13`**

## Entry Point: main.ts

Vapor Mode has a different API from standard Vue. Instead of `createApp()`, we use `createVaporApp()`:

```typescript
import { createVaporApp } from 'vue';
import App from './App.vue';

type VaporRoot = Parameters<typeof createVaporApp>[0];
const RootComponent = App as unknown as VaporRoot;

createVaporApp(RootComponent).mount('#app');
```

**From `whis-desktop/ui/src/main.ts:1-7`**

Key differences:
- `createVaporApp()` expects a different component type (hence the cast)
- No plugins passed at this stage (Tauri API is global, no need for `app.use()`)
- Mount to `#app` in `index.html`, same as regular Vue

The TypeScript cast is necessary because Vapor's types aren't fully stable yet. In Vapor stable, this will just be `createVaporApp(App).mount('#app')`.

## Component Structure

Whis has a simple single-page architecture:

```
App.vue (root)
├── HomeView.vue
├── ShortcutView.vue
├── ApiKeyView.vue
└── AboutView.vue
```

**App.vue** manages:
- Top-level state (settings, backend info, portal shortcut)
- Navigation between views
- Sidebar and window controls

Each view is conditionally rendered with `v-if`:

```vue
<HomeView
  v-if="activeSection === 'home'"
  :current-shortcut="currentShortcut"
  :portal-shortcut="portalShortcut"
/>

<ShortcutView
  v-if="activeSection === 'shortcut'"
  :backend-info="backendInfo"
  :current-shortcut="currentShortcut"
  :portal-shortcut="portalShortcut"
  :portal-bind-error="portalBindError"
  @update:current-shortcut="currentShortcut = $event"
  @update:portal-shortcut="portalShortcut = $event"
/>
```

**From `whis-desktop/ui/src/App.vue:169-183`**

No Vue Router—the app is small enough that conditional rendering is cleaner. `activeSection` is a simple `ref<Section>('home')` that changes on sidebar button clicks.

## Vapor Mode Syntax

All components use `<script setup lang="ts" vapor>`:

```vue
<script setup lang="ts" vapor>
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const status = ref<StatusResponse>({ state: 'Idle', config_valid: false });
const error = ref<string | null>(null);

async function toggleRecording() {
  try {
    await invoke('toggle_recording');
    await fetchStatus();
  } catch (e) {
    error.value = String(e);
  }
}
</script>
```

**From `whis-desktop/ui/src/views/HomeView.vue:1-68`**

The `vapor` attribute tells the Vue compiler to use Vapor Mode compilation. Without it, the component would compile to standard vdom code.

Everything else is normal Vue:
- `ref()` for reactive state
- `computed()` for derived values
- `onMounted()` / `onUnmounted()` for lifecycle
- `defineProps()` / `defineEmits()` for component interface

## Calling Rust Commands

The `@tauri-apps/api` package provides the bridge to Rust. Core function: `invoke()`.

### Basic Invocation

```typescript
import { invoke } from '@tauri-apps/api/core';

const settings = await invoke<Settings>('get_settings');
```

**From `whis-desktop/ui/src/App.vue:49`**

This calls the `get_settings` command we defined in Chapter 21. TypeScript generic `<Settings>` ensures type safety on the returned value.

### With Parameters

```typescript
await invoke<SaveResult>('save_settings', {
  settings: {
    shortcut: currentShortcut,
    provider: provider,
    language: language,
    api_keys: apiKeys
  }
});
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:80-87`**

Parameters are passed as a single object. Keys must match Rust command parameter names (using snake_case).

### Error Handling

```typescript
try {
  await invoke('toggle_recording');
  error.value = null;
} catch (e) {
  error.value = String(e);
}
```

**From `whis-desktop/ui/src/views/HomeView.vue:62-67`**

If the Rust command returns `Err()`, `invoke()` throws. The error message is the string from `Err(String)`.

### Polling Status

HomeView polls the recording state every 500ms:

```typescript
let pollInterval: number | null = null;

async function fetchStatus() {
  try {
    status.value = await invoke<StatusResponse>('get_status');
  } catch (e) {
    console.error('Failed to get status:', e);
  }
}

onMounted(() => {
  fetchStatus();
  pollInterval = window.setInterval(fetchStatus, 500);
});

onUnmounted(() => {
  if (pollInterval) {
    clearInterval(pollInterval);
  }
});
```

**From `whis-desktop/ui/src/views/HomeView.vue:17-79`**

This keeps the UI in sync with the Rust backend. When recording starts (either via button or shortcut), the UI updates within 500ms.

Why polling instead of events? Simplicity. Tauri supports events, but for a small app with infrequent state changes, polling is easier. If Whis needed real-time updates (e.g., live transcription display), events would be better.

## Real-World Flow: Saving Settings

Let's trace what happens when you change a setting and click Save in ApiKeyView:

**1. User edits API key input**

```vue
<input
  type="password"
  :value="getApiKey('openai')"
  @input="updateApiKey('openai', ($event.target as HTMLInputElement).value)"
/>
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:186-190`**

The `@input` handler calls `updateApiKey()`, which emits an event:

```typescript
function updateApiKey(provider: Provider, value: string) {
  const newKeys = { ...props.apiKeys, [provider]: value };
  emit('update:apiKeys', newKeys);
}
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:59-62`**

**2. App.vue receives emit and updates state**

```vue
<ApiKeyView
  :api-keys="apiKeys"
  @update:api-keys="apiKeys = $event"
/>
```

**From `whis-desktop/ui/src/App.vue:190-193`**

The `apiKeys` ref in App.vue is now updated. But settings aren't saved yet—Vue state is ephemeral.

**3. User clicks Save button**

```vue
<button @click="saveSettings" class="btn btn-secondary">Save</button>
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:304`**

This calls the `saveSettings()` function:

```typescript
async function saveSettings() {
  try {
    // Validate keys
    const openaiKey = props.apiKeys.openai || '';
    if (openaiKey && !openaiKey.startsWith('sk-')) {
      status.value = "Invalid OpenAI key format";
      return;
    }

    await invoke<SaveResult>('save_settings', {
      settings: {
        shortcut: props.currentShortcut,
        provider: props.provider,
        language: props.language,
        api_keys: props.apiKeys
      }
    });
    status.value = "Saved";
    setTimeout(() => status.value = "", 2000);
  } catch (e) {
    status.value = "Failed to save: " + e;
  }
}
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:64-93`**

**4. Rust command writes to disk**

The `save_settings` command (from Chapter 21) serializes the settings to JSON and writes to `~/.config/whis/settings.json`.

**5. UI shows feedback**

The `status` ref is updated to "Saved", which triggers Vapor's reactivity system to update the DOM. After 2 seconds, it clears.

```vue
<div class="status" :class="{ visible: status }">{{ status }}</div>
```

**From `whis-desktop/ui/src/views/ApiKeyView.vue:306`**

The `visible` class is conditionally applied, which CSS transitions handle for fade-in/out effect.

## State Management: No Store Needed

Whis doesn't use Pinia or Vuex. Why?

1. **Small scope**: Only 4 views, shared state fits in App.vue
2. **Rust is the source of truth**: Settings live in `~/.config/whis/settings.json`, recording state in `AppState.recording_state`
3. **Props/emits are enough**: Parent-child communication is simple with v-model pattern

App.vue holds top-level reactive refs:

```typescript
const currentShortcut = ref("Ctrl+Shift+R");
const portalShortcut = ref<string | null>(null);
const provider = ref<'openai' | 'mistral' | 'groq'>('openai');
const language = ref<string | null>(null);
const apiKeys = ref<Record<string, string>>({});
const backendInfo = ref<BackendInfo | null>(null);
```

**From `whis-desktop/ui/src/App.vue:30-36`**

On mount, `loadSettings()` fetches from Rust:

```typescript
async function loadSettings() {
  try {
    const settings = await invoke<Settings>('get_settings');
    currentShortcut.value = settings.shortcut;
    provider.value = settings.provider || 'openai';
    language.value = settings.language;
    apiKeys.value = settings.api_keys || {};
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}
```

**From `whis-desktop/ui/src/App.vue:47-57`**

These refs are passed as props to child views. Child views emit updates, App.vue handles them. This keeps data flow unidirectional and easy to trace.

## Window Controls & Wayland Dragging

Tauri apps on Linux can use GTK's native decorations or custom controls. Whis uses custom controls for consistency:

```vue
<div v-if="showCustomControls" class="titlebar" data-tauri-drag-region>
  <div class="window-controls">
    <button class="control-btn" @click="minimizeWindow" title="Minimize">
      <svg>...</svg>
    </button>
    <button class="control-btn close" @click="closeWindow" title="Close">
      <svg>...</svg>
    </button>
  </div>
</div>
```

**From `whis-desktop/ui/src/App.vue:157-166`**

The `data-tauri-drag-region` attribute is crucial for Wayland. It marks the titlebar as draggable, allowing the user to move the window. Without it, the window would be stuck in place.

On Wayland, this works because of the GTK titlebar fix we set in Rust (`main.rs:6-13`):

```rust
gtk::glib::set_prgname(Some("ink.whis.Whis"));
```

This ensures Wayland recognizes the window properly and respects `data-tauri-drag-region`.

Close button behavior:

```typescript
async function closeWindow() {
  try {
    const canReopen = await invoke<boolean>('can_reopen_window');
    if (canReopen) {
      await getCurrentWindow().hide();
    } else {
      await getCurrentWindow().close();
    }
  } catch (e) {
    await getCurrentWindow().close();
  }
}
```

**From `whis-desktop/ui/src/App.vue:64-77`**

If shortcuts are working or tray icon is available, we **hide** the window instead of closing. The user can reopen it with the shortcut or tray menu. If neither works, we actually quit—otherwise the user would be stranded.

## Styling: Design System

Whis uses a consistent design system with CSS custom properties:

```css
:root {
  --bg: hsl(0, 0%, 7%);
  --bg-weak: hsl(0, 0%, 11%);
  --bg-hover: hsl(0, 0%, 16%);
  --text: hsl(0, 0%, 80%);
  --text-weak: hsl(0, 0%, 62%);
  --accent: hsl(48, 100%, 60%);  /* Gold */
  --border: hsl(0, 0%, 24%);
  --font: "JetBrains Mono", "Fira Code", monospace;
}
```

**From `whis-desktop/ui/src/App.vue:209-237`**

Design matches the whis.ink website:
- **Dark theme**: `--bg` is almost black (7% lightness)
- **Gold accent**: `hsl(48, 100%, 60%)` for highlights
- **Monospace font**: JetBrains Mono for that terminal aesthetic

All buttons, inputs, and components reference these tokens. Changing the theme is a matter of updating `:root`.

Shared styles defined in `<style>` (not scoped):

```css
.btn {
  padding: 10px 20px;
  background: var(--bg-strong);
  border: none;
  border-radius: 4px;
  font-size: 12px;
  color: var(--text-inverted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.btn:hover:not(:disabled) {
  background: hsl(0, 0%, 90%);
}
```

**From `whis-desktop/ui/src/App.vue:318-342`**

View-specific styles use `<style scoped>` to prevent leakage.

## Build Process: Vite & Rolldown

Whis uses **Vite** (technically, rolldown-vite) for development and bundling:

```json
"scripts": {
  "dev": "vite",
  "build": "vue-tsc -b && vite build",
  "preview": "vite preview"
}
```

**From `whis-desktop/ui/package.json:5-9`**

Note the Vite override:

```json
"vite": "npm:rolldown-vite@7.2.8",
"overrides": {
  "vite": "npm:rolldown-vite@7.2.8"
}
```

**From `whis-desktop/ui/package.json:21-26`**

**Rolldown** is Vite's upcoming Rust-based bundler (replacement for esbuild/rollup). It's even faster and better integrated with Vite 7+. Again, bleeding-edge choice for a hobby project.

Vite config is minimal:

```typescript
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
```

**From `whis-desktop/ui/vite.config.ts:4-17`**

- `clearScreen: false`: Prevents Vite from clearing terminal (nice when running alongside Rust logs)
- `strictPort: true`: Fail if port 5173 is taken (Tauri expects this exact port)
- `envPrefix`: Vite injects `VITE_*` and `TAURI_*` env vars into the build
- `target: es2021`: Modern browsers only, no legacy polyfills
- `minify`: Only in release builds

During development:

```bash
cd crates/whis-desktop/ui
npm run dev
```

Vite starts at `http://localhost:5173`. Tauri's dev server loads this URL and injects the API bridge.

For production build:

```bash
cd crates/whis-desktop/ui
npm run build
```

Vite compiles to `ui/dist/`. Tauri's build process (in `build.rs`) copies this into the final binary.

## Component Deep Dive: ShortcutView

Let's examine the most complex view—ShortcutView—which handles three different shortcut backends:

### Backend Detection

On mount, ShortcutView receives `backendInfo` prop from App.vue:

```typescript
const props = defineProps<{
  backendInfo: BackendInfo | null;
  currentShortcut: string;
  portalShortcut: string | null;
  portalBindError: string | null;
}>();
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:17-22`**

The template has three sections based on backend:

```vue
<!-- Portal backend (Wayland) -->
<template v-if="backendInfo?.backend === 'PortalGlobalShortcuts'">
  <!-- Portal-specific UI -->
</template>

<!-- Manual Setup (Wayland without portal) -->
<template v-else-if="backendInfo?.backend === 'ManualSetup'">
  <!-- Manual config instructions -->
</template>

<!-- Tauri plugin (X11/macOS/Windows) -->
<template v-else>
  <!-- Simple key capture -->
</template>
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:186-354`**

### Key Capture (X11/macOS)

For TauriPlugin backend, users can capture keys directly:

```vue
<div
  class="shortcut-input"
  :class="{ recording: isRecording }"
  @click="startRecording"
  @blur="stopRecording"
  @keydown="handleKeyDown"
  tabindex="0"
>
  <div class="keys">
    <span v-for="(key, index) in shortcutKeys" :key="index" class="key">
      {{ key }}
    </span>
  </div>
  <span v-if="isRecording" class="recording-dot"></span>
</div>
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:325-342`**

Click activates recording mode. The `handleKeyDown` listener captures the key combo:

```typescript
function handleKeyDown(e: KeyboardEvent) {
  if (!isRecording.value) return;
  e.preventDefault();

  const keys = [];
  if (e.ctrlKey) keys.push('Ctrl');
  if (e.shiftKey) keys.push('Shift');
  if (e.altKey) keys.push('Alt');
  if (e.metaKey) keys.push('Super');

  const key = e.key.toUpperCase();
  if (!['CONTROL', 'SHIFT', 'ALT', 'META'].includes(key)) {
    keys.push(key);
  }

  if (keys.length > 0) {
    emit('update:currentShortcut', keys.join('+'));
  }
}
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:146-164`**

This builds a string like `"Ctrl+Shift+R"`, which is emitted back to App.vue.

### Portal Configuration (Wayland/GNOME)

For Portal backend, the flow is different:

1. **Capture key combo** (same as above)
2. **Click Apply** button
3. **Call `configure_shortcut_with_trigger` command**
4. **Wait for portal dialog** (GNOME shows system settings)
5. **Portal returns actual binding** (may differ if conflict)

```typescript
async function configureWithCapturedKey() {
  try {
    status.value = "Configuring...";
    const newBinding = await invoke<string | null>('configure_shortcut_with_trigger', {
      trigger: props.currentShortcut
    });
    if (newBinding) {
      emit('update:portalShortcut', newBinding);
      status.value = "Configured!";
    } else {
      status.value = "Cancelled";
    }
  } catch (e) {
    status.value = "Failed: " + e;
  }
}
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:119-140`**

If successful, `portalShortcut` is updated and the UI switches to "bound" mode, showing the actual shortcut and a Reset button.

### Manual Setup Instructions (Sway/Hyprland)

For ManualSetup backend, ShortcutView displays compositor-specific instructions:

```vue
<template v-if="backendInfo.compositor.toLowerCase().includes('sway')">
  <p class="hint">Add to <code>~/.config/sway/config</code>:</p>
  <div class="command">
    <code>bindsym {{ currentShortcut.toLowerCase() }} exec {{ toggleCommand }}</code>
  </div>
</template>
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:296-301`**

The `toggleCommand` is fetched on mount:

```typescript
onMounted(async () => {
  try {
    toggleCommand.value = await invoke<string>('get_toggle_command');
  } catch (e) {
    console.error('Failed to get toggle command:', e);
  }
});
```

**From `whis-desktop/ui/src/views/ShortcutView.vue:34-40`**

This returns platform-specific command (on Linux: `whis-desktop --toggle`, on macOS: `/Applications/Whis.app/Contents/MacOS/whis-desktop --toggle`).

## Testing the UI

During development, test the UI in isolation:

```bash
cd crates/whis-desktop/ui
npm run dev
```

Visit `http://localhost:5173`. You'll see the UI, but Tauri commands will fail (no Rust backend). To test with Rust:

```bash
cd crates/whis-desktop
cargo tauri dev
```

This compiles Rust, starts Vite, and launches the app. Hot reload works for both Rust (needs recompile) and Vue (instant HMR).

## Summary

**Key Takeaways:**

1. **Vapor Mode**: Vue 3.6 alpha with no Virtual DOM—compiled reactivity for smaller bundles
2. **Component structure**: App.vue manages top-level state, views handle UI logic
3. **Tauri API**: `invoke()` calls Rust commands with full type safety
4. **No store needed**: Props/emits handle state, Rust is source of truth
5. **Platform awareness**: UI adapts to backend (TauriPlugin/Portal/Manual)
6. **Vite + Rolldown**: Fast dev server and Rust-powered bundling

**Where This Matters in Whis:**

- ShortcutView shows different UI based on platform capabilities
- HomeView polls recording state to keep UI in sync
- ApiKeyView validates keys before calling Rust
- Window controls handle platform-specific decoration needs

**Patterns Used:**

- **Props down, events up**: Standard Vue pattern for data flow
- **Conditional rendering**: Different sections based on backend
- **Polling**: Simple state sync for infrequent updates
- **Computed properties**: Derived state for shortcut formatting

**Design Decisions:**

1. **Why Vapor Mode?** Hobby project, chance to experiment with Vue's future.
2. **Why no Vue Router?** Small app, conditional rendering is simpler.
3. **Why no Pinia?** Rust backend is source of truth, no need for client store.
4. **Why polling over events?** Simplicity for infrequent state changes.
5. **Why rolldown-vite?** Faster builds, better Vite 7+ integration.

This completes Part VII: Vue Frontend. Next, we'll explore common patterns and best practices across the entire Whis codebase.

---

Next: [Chapter 24: Common Patterns in Whis](../part8-patterns/ch24-patterns.md)
