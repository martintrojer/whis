# Part VIII: Patterns & Philosophy

With the entire codebase explored, let's step back and analyze the **design decisions** and **alternatives**.

## What You'll Learn

This part is about **understanding the "why"**:

- **Chapter 24: Design Patterns** - Identifying patterns used (Strategy, Singleton, State Machine)
- **Chapter 25: Alternative Designs** - What else could have been done and tradeoffs
- **Chapter 26: Extending Whis** - Practical guides for adding features

## Why This Matters

Understanding design decisions helps you:

1. **Reason from first principles** - Why this architecture?
2. **Make informed changes** - When to follow existing patterns vs. break them
3. **Extend confidently** - Add features that fit the architecture

```admonish info
This part assumes you've read Parts I-VII and have a complete mental model of the codebase. We'll reference code throughout.
```

## The Meta-Level

Instead of "how does this work?", we ask:

- **Why trait objects instead of enums?** (Chapter 24)
- **Could we use actors instead of Mutex?** (Chapter 25)
- **How do I add a new provider?** (Chapter 26)

## Time Estimate

- **Quick read**: ~45 minutes
- **Thorough read with reflection**: ~2 hours
- **With extension exercises**: +2 hours

---

Let's analyze [Chapter 24: Design Patterns in Whis](./ch24-patterns.md).
