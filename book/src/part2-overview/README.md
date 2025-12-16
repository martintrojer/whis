# Part II: Whis from 30,000 Feet

Now that we've refreshed Rust fundamentals, let's look at Whis from a high level before diving into implementation details.

## What You'll Learn

This part provides the **big picture** without code deep-dives:

- **Chapter 6: What Whis Does** - The user story and data flow
- **Chapter 7: Crate Architecture** - The workspace structure and why
- **Chapter 8: Feature Flags** - Conditional compilation strategy

## Why This Overview?

Before examining individual modules, you need to understand:

1. **The problem Whis solves** - Voice-to-text transcription with multiple providers
2. **The architecture decisions** - Why split into `whis-core`, `whis-cli`, `whis-desktop`, `whis-mobile`
3. **The feature system** - How platform-specific code is managed

```admonish info
Think of this part as reading a map before starting a journey. You'll reference back to these chapters as you explore specific modules.
```

## Time Estimate

- **Quick read**: ~20 minutes
- **Thorough read with diagrams**: ~45 minutes

---

Let's start with [Chapter 6: What Whis Does](./ch06-what-whis-does.md).
