# Part IV: The Core Library - Advanced

Building on the basics, this part explores the **sophisticated patterns** in `whis-core`.

## What You'll Learn

These chapters dive into advanced Rust patterns and async programming:

- **Chapter 13: Audio Encoding** - FFmpeg integration, LAME encoder, chunking algorithm
- **Chapter 14: The Provider System** - Trait objects, dynamic dispatch, extensibility
- **Chapter 15: Parallel Transcription** - Tokio concurrency, semaphores, overlap merging

## Why Advanced?

These modules demonstrate:

1. **Strategy pattern** - Pluggable transcription providers via traits
2. **Async/await in practice** - Parallel API requests with rate limiting
3. **Platform-specific code** - Feature flags for desktop vs mobile

```admonish warning
This part assumes you've read Part I (Rust Refresher) and Part III (Core Easy). We'll use concepts like `Arc<dyn Trait>`, `tokio::spawn`, and conditional compilation extensively.
```

## The Heart of Whis

This part covers the **core engine**:

- How providers abstract different transcription APIs
- How large recordings are chunked and processed in parallel
- How overlap between chunks is intelligently merged

## Time Estimate

- **Quick read**: ~1.5 hours
- **Thorough read with code exploration**: ~3-4 hours
- **With exercises**: +1.5 hours

---

Let's dive into [Chapter 13: Audio Encoding](./ch13-encoding.md).
