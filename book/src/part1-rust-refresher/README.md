# Part I: Rust Refresher

Before diving into the Whis codebase, let's refresh the Rust fundamentals you'll encounter throughout the project.

## What You'll Learn

This part covers **pure Rust concepts** with no Whis-specific code. Each chapter focuses on patterns you'll see repeatedly in the codebase:

- **Chapter 1: Ownership & Borrowing** - The foundation: move semantics, borrowing rules, and interior mutability
- **Chapter 2: Smart Pointers** - `Box`, `Rc`, `Arc`, and when to use each
- **Chapter 3: Traits & Generics** - Static vs dynamic dispatch, trait objects, and bounds
- **Chapter 4: Error Handling** - `Result`, `anyhow`, and error propagation
- **Chapter 5: Async Rust** - Futures, tokio runtime, and spawn patterns

## Why Start Here?

Even if you're experienced with Rust, these chapters serve as:

1. **Common vocabulary** - We'll use these terms throughout the book
2. **Pattern reference** - Quick refreshers when you encounter these in Whis
3. **Bottom-up foundation** - Understanding primitives before seeing them in complex contexts

```admonish tip
If you're very comfortable with a chapter's topic, feel free to skim it. But note the specific patterns highlighted—they appear in Whis.
```

## Time Estimate

- **Quick skim** (if you're comfortable): ~30 minutes
- **Thorough read** (refresher needed): ~2-3 hours
- **With exercises**: +1 hour

---

Ready? Let's start with [Chapter 1: Ownership & Borrowing](./ch01-ownership.md).
