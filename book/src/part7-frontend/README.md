# Part VII: Vue Frontend

This part covers the Vue 3 frontend that powers the `whis-desktop` UI.

## What You'll Learn

Since you're comfortable with Vue, this part is concise:

- **Chapter 23: Frontend Integration** - Project structure, invoking Rust commands, reactive state

## The Frontend's Role

The Vue app in `crates/whis-desktop/ui/` provides:

1. **Settings UI** - Configure provider, API keys, shortcuts
2. **Recording controls** - Start/stop button with visual feedback
3. **Status display** - Current recording state

```admonish tip
The frontend is straightforward Vue 3 with Composition API. The interesting part is how it communicates with Rust via `invoke()`.
```

## Code Organization

Files in `crates/whis-desktop/ui/src/`:

- `main.ts` - App initialization
- `App.vue` - Main layout and navigation
- `views/*.vue` - Individual pages (Home, Shortcuts, API Keys, About)

## Time Estimate

- **Quick read**: ~20 minutes
- **Thorough read with code exploration**: ~45 minutes

---

Let's explore [Chapter 23: Frontend Integration](./ch23-vue.md).
