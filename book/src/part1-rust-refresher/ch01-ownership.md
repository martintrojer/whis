# Chapter 1: Ownership & Borrowing Refresher

Ownership is Rust's most distinctive feature—it's what enables memory safety without garbage collection. Before we see how Whis uses ownership patterns, let's refresh the fundamentals.

## The Three Rules of Ownership

Rust's ownership system is built on three rules:

1. **Each value has a single owner** (variable binding)
2. **When the owner goes out of scope, the value is dropped**
3. **There can only be ONE owner at a time**

These rules seem simple, but their implications are profound.

## Move Semantics

Let's start with the most common ownership pattern you'll see in Whis:

```rust
fn main() {
    let s1 = String::from("hello");
    let s2 = s1;  // s1 is MOVED to s2
    
    // println!("{}", s1);  // ERROR! s1 is no longer valid
    println!("{}", s2);     // OK
}
```

What happened here?

1. `s1` owns the String data on the heap
2. `s2 = s1` **transfers ownership** (moves) from s1 to s2
3. `s1` is now invalid and cannot be used
4. When `s2` goes out of scope, the String is dropped

> **Why move instead of copy?** Copying heap data is expensive. Rust prefers explicit cloning.

### When Does Rust Move?

```rust
fn take_ownership(s: String) {
    println!("{}", s);
}  // s is dropped here

fn main() {
    let my_string = String::from("test");
    take_ownership(my_string);  // my_string is MOVED into function
    
    // println!("{}", my_string);  // ERROR! Value was moved
}
```

**Rule of thumb**: If a type doesn't implement `Copy`, passing it by value **moves** it.

### Types That Copy Instead of Move

Some types implement the `Copy` trait and are copied bitwise instead of moved:

```rust
fn main() {
    let x = 5;
    let y = x;  // x is COPIED (not moved)
    
    println!("x = {}, y = {}", x, y);  // Both valid!
}
```

**Copy types** include:
- All integer types (`i32`, `u64`, etc.)
- Boolean (`bool`)
- Floating point types (`f32`, `f64`)
- Character (`char`)
- Tuples containing only `Copy` types: `(i32, i32)`

**Non-Copy types** include:
- `String` (owns heap data)
- `Vec<T>` (owns heap allocation)
- Most structs (unless you explicitly `derive(Copy)`)

## Borrowing: References to Data

Instead of transferring ownership, you can **borrow** a value with references:

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope, but it doesn't own the String, so nothing is dropped

fn main() {
    let my_string = String::from("hello");
    let len = calculate_length(&my_string);  // Borrow with &
    
    println!("'{}' has length {}", my_string, len);  // my_string still valid!
}
```

The `&` creates a **reference** that borrows the value without taking ownership.

### Immutable vs Mutable References

Rust enforces strict borrowing rules at compile time:

```rust
fn main() {
    let mut s = String::from("hello");
    
    // Immutable borrow
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);  // OK: multiple immutable borrows
    
    // Mutable borrow
    let r3 = &mut s;
    r3.push_str(" world");
    println!("{}", r3);  // OK
}
```

**The Borrowing Rules:**

1. **Any number of immutable references** (`&T`) OR
2. **Exactly ONE mutable reference** (`&mut T`)
3. **References must always be valid** (no dangling references)

### Why These Rules?

They prevent **data races** at compile time:

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s;        // immutable borrow
    let r2 = &mut s;    // ERROR! Cannot borrow as mutable while immutable borrow exists
    
    println!("{}", r1);
}
```

If both `r1` and `r2` were allowed, `r2` could modify the String while `r1` is reading it—a classic data race.

### Non-Lexical Lifetimes (NLL)

Modern Rust is smart about when borrows end:

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s;
    let r2 = &s;
    println!("{} and {}", r1, r2);
    // r1 and r2 are no longer used after this point
    
    let r3 = &mut s;  // OK! Immutable borrows ended
    r3.push_str(" world");
    println!("{}", r3);
}
```

The borrow checker understands that `r1` and `r2` aren't used after the `println!`, so `r3` is safe.

## Lifetimes: Explicit Borrow Scopes

Sometimes Rust can't infer how long references should be valid. You must annotate **lifetimes**:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

The `'a` lifetime parameter says:

> "The returned reference will be valid for as long as both `x` and `y` are valid."

### Reading Lifetime Annotations

```rust
&i32        // a reference
&'a i32     // a reference with an explicit lifetime
&'a mut i32 // a mutable reference with an explicit lifetime
```

You'll see lifetimes in Whis when:
- Functions return references
- Structs hold references
- Trait bounds involve references

### The `'static` Lifetime

The special lifetime `'static` means "lives for the entire program":

```rust
let s: &'static str = "I live forever";
```

String literals are `'static` because they're embedded in the binary. You'll see `'static` in Whis for:
- Provider names: `fn name(&self) -> &'static str`
- Static constants

## Interior Mutability: Breaking the Rules (Safely)

What if you need to mutate data through an immutable reference? Rust provides **interior mutability** patterns:

### `RefCell<T>` - Single-Threaded Interior Mutability

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(5);
    
    // Immutable reference to RefCell
    let r = &data;
    
    // But we can mutate the interior!
    *r.borrow_mut() += 1;
    
    println!("data = {}", r.borrow());  // 6
}
```

`RefCell<T>` enforces borrowing rules **at runtime** instead of compile time:

```rust
use std::cell::RefCell;

fn main() {
    let data = RefCell::new(5);
    
    let r1 = data.borrow();     // OK: immutable borrow
    let r2 = data.borrow_mut(); // PANIC at runtime! Already borrowed
}
```

> **When to use `RefCell`**: Single-threaded code where you need mutation through shared references.

### `Mutex<T>` - Thread-Safe Interior Mutability

For multi-threaded code, use `Mutex<T>`:

```rust
use std::sync::Mutex;

fn main() {
    let data = Mutex::new(5);
    
    {
        let mut guard = data.lock().unwrap();
        *guard += 1;
    }  // Lock released when guard is dropped
    
    println!("data = {:?}", data);
}
```

`Mutex<T>` provides:
- **Interior mutability** (mutate through `&Mutex`)
- **Thread safety** (only one thread can hold the lock)

You'll see this **constantly** in Whis combined with `Arc`:

```rust
use std::sync::{Arc, Mutex};

let shared_data = Arc::new(Mutex::new(Vec::new()));
```

This pattern allows:
- Multiple owners (`Arc`)
- Mutation through shared reference (`Mutex`)
- Thread safety (both are `Send + Sync`)

We'll explore this deeply in the next chapter on smart pointers.

## Common Patterns in Whis

Now that we've refreshed the fundamentals, here are ownership patterns you'll encounter in Whis:

### Pattern 1: Taking Ownership for Transformation

```rust
// In whis-core/src/audio.rs
pub fn finalize_recording(mut self) -> Result<RecordingOutput> {
    // Takes ownership of self, consumes it, returns transformed data
}
```

**Why take ownership?** After finalizing, the recorder is invalid. Taking ownership prevents reuse.

### Pattern 2: Borrowing for Inspection

```rust
// In whis-core/src/provider/mod.rs
fn transcribe_sync(&self, api_key: &str, request: TranscriptionRequest) -> Result<TranscriptionResult>
```

**Why borrow `api_key`?** We only need to read it, not own it.

### Pattern 3: Mutable Borrow for Modification

```rust
// In whis-core/src/audio.rs
pub fn start_recording(&mut self) -> Result<()> {
    // Needs &mut self to modify internal state
}
```

**Why `&mut self`?** We're starting the recording, which changes the recorder's state.

### Pattern 4: `std::mem::take` - Ownership Trick

```rust
use std::mem;

let samples: Vec<f32> = {
    let mut guard = self.samples.lock().unwrap();
    mem::take(&mut *guard)  // Takes ownership, leaves empty Vec
};
```

**What's happening?**
1. `guard` is a mutable lock guard
2. `mem::take` **moves out** the Vec, replacing it with an empty one
3. This satisfies the borrow checker without cloning

You'll see this in `whis-core/src/audio.rs:127-130`.

## Summary

**Key Takeaways:**

1. **Ownership = single owner**, value dropped when owner goes out of scope
2. **Move semantics** transfer ownership (default for non-Copy types)
3. **Borrowing** lets you access data without owning it
4. **Borrowing rules** prevent data races: many `&T` XOR one `&mut T`
5. **Lifetimes** explicitly track how long references are valid
6. **Interior mutability** (`RefCell`, `Mutex`) allows mutation through shared refs

**Where This Matters in Whis:**

- `Arc<Mutex<Vec<f32>>>` in the audio recorder (Chapter 2)
- `Arc<dyn TranscriptionBackend>` in the provider registry (Chapter 3)
- Lifetime annotations in provider traits
- `mem::take` pattern for extracting data from mutexes

## Exercise

Try to compile this code and fix the errors:

```rust
fn main() {
    let mut s = String::from("hello");
    
    let r1 = &s;
    let r2 = &mut s;
    
    r2.push_str(" world");
    println!("{}", r1);
}
```

**Questions:**
1. What error does the compiler give?
2. Why is this error important for safety?
3. How would you fix it?

**Hint:** Think about when borrows end.

---

Next: [Chapter 2: Smart Pointers](./ch02-smart-pointers.md)
