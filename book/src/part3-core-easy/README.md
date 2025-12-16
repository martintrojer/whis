# Part III: The Core Library - Easy

Now we dive into actual Whis code, starting with the **simplest modules** in `whis-core`.

## What You'll Learn

This part covers straightforward modules that don't require advanced Rust patterns:

- **Chapter 9: Configuration** - Reading/writing settings with serde
- **Chapter 10: Clipboard** - Cross-platform clipboard operations
- **Chapter 11: Audio Basics** - Understanding the cpal audio pipeline
- **Chapter 12: Audio Lifecycle** - Recording start, stop, and data extraction

## Why Start Easy?

These modules demonstrate core functionality without overwhelming complexity:

1. **Settings** - Simple file I/O and JSON serialization
2. **Clipboard** - Platform abstraction without heavy concurrency
3. **Audio basics** - Understanding the cpal model before encoding/chunking

```admonish tip
By the end of this part, you'll understand how Whis captures audio from the microphone. The next part covers what happens to that audio data.
```

## Code Organization

All code in this part lives in `crates/whis-core/src/`:

- `settings.rs` - Configuration management
- `clipboard.rs` - Clipboard operations  
- `audio.rs` - Audio recording (basic and lifecycle)

## Time Estimate

- **Quick read**: ~1 hour
- **Thorough read with code exploration**: ~2-3 hours
- **With exercises**: +1 hour

---

Let's begin with [Chapter 9: Configuration & Settings](./ch09-configuration.md).
