# Chapter 2: Smart Pointers

Smart pointers are types that act like pointers but have additional metadata and capabilities. They're crucial for understanding Whis's memory management patterns.

## What Makes a Pointer "Smart"?

Regular references (`&T`) are just addresses—they borrow but don't own. **Smart pointers**:

1. **Own** the data they point to
2. Implement `Deref` (act like references)
3. Implement `Drop` (clean up when dropped)
4. Often provide additional guarantees (reference counting, thread safety)

The three you'll see constantly in Whis: `Box<T>`, `Rc<T>`, and `Arc<T>`.

## `Box<T>` - Heap Allocation

The simplest smart pointer: `Box<T>` puts data on the heap.

```rust
fn main() {
    let b = Box::new(5);
    println!("b = {}", b);
}  // b and the heap data are dropped
```

### When to Use `Box`?

1. **Large data** - Avoid stack overflow by heap-allocating
2. **Trait objects** - `Box<dyn Trait>` for dynamic dispatch
3. **Recursive types** - Break infinite size loops

```rust
// Without Box, this has infinite size!
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

### `Box` in Whis

You won't see much bare `Box` in Whis because:
- Most heap allocations use `String` or `Vec` (which internally use `Box`-like allocation)
- Trait objects use `Arc<dyn Trait>` instead of `Box<dyn Trait>` (for sharing)

## `Rc<T>` - Reference Counting (Single-Threaded)

`Rc<T>` enables **multiple ownership** through reference counting:

```rust
use std::rc::Rc;

fn main() {
    let a = Rc::new(String::from("hello"));
    let b = Rc::clone(&a);  // Increment ref count
    let c = Rc::clone(&a);  // Increment ref count
    
    println!("Reference count: {}", Rc::strong_count(&a));  // 3
}  // c dropped, count = 2
   // b dropped, count = 1
   // a dropped, count = 0, String is freed
```

### How `Rc` Works

- Stores data + reference count on the heap
- `Rc::clone()` increments count (cheap!)
- When count reaches 0, data is freed

```rust
use std::rc::Rc;

let a: Rc<String> = Rc::new(String::from("shared"));
let b = Rc::clone(&a);

// Both a and b point to the same data
println!("{}", a);  // "shared"
println!("{}", b);  // "shared"
```

> **Key Point**: `Rc::clone()` does NOT clone the data, just the pointer!

### `Rc` Limitations

1. **Not thread-safe** - Only use in single-threaded code
2. **Immutable only** - `Rc<T>` provides `&T`, not `&mut T`

For thread safety, use `Arc<T>`. For mutability, combine with `RefCell<T>` or `Mutex<T>`.

### Why Not Use `Rc` in Whis?

Whis is multi-threaded (async audio recording, parallel transcription). `Rc<T>` is not `Send`, so it can't cross thread boundaries. We use `Arc<T>` instead.

## `Arc<T>` - Atomic Reference Counting (Thread-Safe)

`Arc<T>` is `Rc<T>`'s thread-safe sibling:

```rust
use std::sync::Arc;
use std::thread;

fn main() {
    let data = Arc::new(vec![1, 2, 3]);
    
    let data_clone = Arc::clone(&data);
    let handle = thread::spawn(move || {
        println!("Thread: {:?}", data_clone);
    });
    
    println!("Main: {:?}", data);
    handle.join().unwrap();
}
```

### `Arc` vs `Rc`

| Feature | `Rc<T>` | `Arc<T>` |
|---------|---------|----------|
| Thread-safe? | ❌ No | ✅ Yes |
| Performance | Faster (no atomics) | Slower (atomic ops) |
| Use when | Single-threaded | Multi-threaded |
| Implements | `!Send`, `!Sync` | `Send`, `Sync` |

**Rule of thumb**: Default to `Arc` in async code, use `Rc` only if you're certain it's single-threaded.

### `Arc` in Whis

`Arc` appears **everywhere** in Whis:

```rust
// whis-core/src/audio.rs:40-44
pub struct AudioRecorder {
    samples: Arc<Mutex<Vec<f32>>>,  // Shared across threads
    sample_rate: u32,
    channels: u16,
    stream: Option<cpal::Stream>,
}
```

Why `Arc<Mutex<Vec<f32>>>`?

1. **`Vec<f32>`** - The audio samples
2. **`Mutex`** - Interior mutability (audio callback mutates while we read)
3. **`Arc`** - Shared between recorder thread and audio callback

## The `Arc<Mutex<T>>` Pattern

This is the **most common pattern** for shared mutable state in Rust:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Result: {}", *counter.lock().unwrap());  // 10
}
```

### Breaking It Down

1. **`Arc`** - Multiple threads own the counter
2. **`Mutex`** - Only one thread can access at a time
3. **`.lock()`** - Acquires the lock, blocks if held by another thread
4. **`.unwrap()`** - Handle potential poisoning (we'll cover this)
5. **`*num += 1`** - Dereference the guard to mutate the value

### Lock Guards and Drop

```rust
use std::sync::{Arc, Mutex};

let data = Arc::new(Mutex::new(5));

{
    let mut guard = data.lock().unwrap();
    *guard += 1;
}  // Lock automatically released when guard is dropped

// Now another thread can acquire the lock
```

The `MutexGuard` returned by `.lock()` implements `Drop`:

```rust
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Release the lock
    }
}
```

**Best practice**: Keep lock guards in small scopes to minimize contention.

### Mutex Poisoning

If a thread panics while holding a lock, the `Mutex` is **poisoned**:

```rust
use std::sync::Mutex;

let data = Mutex::new(0);

let result = std::panic::catch_unwind(|| {
    let mut guard = data.lock().unwrap();
    *guard = 42;
    panic!("oops");  // Lock is poisoned
});

// Next lock() returns Err
match data.lock() {
    Ok(guard) => println!("Got lock: {}", *guard),
    Err(poisoned) => {
        println!("Mutex was poisoned!");
        let guard = poisoned.into_inner();  // Can still access data
        println!("Data: {}", *guard);  // 42
    }
}
```

In Whis, we usually `.unwrap()` because:
- If a critical thread panics, we want the whole app to crash
- Mutex poisoning indicates a serious bug

## When to Clone an `Arc`

`Arc::clone()` is cheap (atomic increment), but **when should you clone**?

### Pattern 1: Moving into Closures

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

// Clone before moving into closure
let data_clone = Arc::clone(&data);
let closure = move || {
    println!("{:?}", data_clone);
};
```

### Pattern 2: Spawning Tasks

```rust
use std::sync::Arc;
use tokio::task;

let shared = Arc::new(String::from("hello"));

for i in 0..5 {
    let shared = Arc::clone(&shared);  // Clone for each task
    task::spawn(async move {
        println!("Task {}: {}", i, shared);
    });
}
```

### Pattern 3: Storing in Structs

```rust
use std::sync::Arc;

struct Worker {
    data: Arc<Vec<i32>>,  // Holds an Arc
}

impl Worker {
    fn new(data: Arc<Vec<i32>>) -> Self {
        Worker { data }  // No clone needed if we consume the Arc
    }
}
```

## `Arc<dyn Trait>` - Shared Trait Objects

Combining `Arc` with trait objects enables polymorphism with shared ownership:

```rust
use std::sync::Arc;

trait Shape {
    fn area(&self) -> f64;
}

struct Circle { radius: f64 }
impl Shape for Circle {
    fn area(&self) -> f64 { 3.14 * self.radius * self.radius }
}

fn main() {
    let shapes: Vec<Arc<dyn Shape>> = vec![
        Arc::new(Circle { radius: 2.0 }),
    ];
    
    for shape in &shapes {
        println!("Area: {}", shape.area());
    }
}
```

### In Whis's Provider Registry

```rust
// whis-core/src/provider/mod.rs:180-181
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}
```

Why `Arc<dyn TranscriptionBackend>` instead of `Box<dyn TranscriptionBackend>`?

1. **Cloning** - We can clone the Arc to give multiple owners
2. **Thread safety** - Providers are used across async tasks
3. **Efficiency** - No need to duplicate provider structs

## `Weak<T>` - Breaking Cycles

`Arc` creates reference cycles that leak memory:

```rust
use std::sync::Arc;

struct Node {
    next: Option<Arc<Node>>,
}

// This leaks: A -> B -> A
let a = Arc::new(Node { next: None });
let b = Arc::new(Node { next: Some(Arc::clone(&a)) });
// Can't set a.next = b because Node is immutable
```

`Weak<T>` breaks cycles by not incrementing the strong count:

```rust
use std::sync::{Arc, Weak};

struct Node {
    next: Option<Weak<Node>>,  // Weak pointer
}

let a = Arc::new(Node { next: None });
let weak_a = Arc::downgrade(&a);  // Create Weak
```

**You won't see `Weak` much in Whis** because:
- Most structures are acyclic (tree-like, not graph-like)
- Short-lived objects don't leak significantly

## Practical Example: Whis Audio Recorder

Let's see how `Arc<Mutex<T>>` is used in real Whis code:

```rust
// whis-core/src/audio.rs:46-54
pub fn new() -> Result<Self> {
    Ok(AudioRecorder {
        samples: Arc::new(Mutex::new(Vec::new())),  // Shared mutable buffer
        sample_rate: 44100,
        channels: 1,
        stream: None,
    })
}
```

Later, when starting recording:

```rust
// whis-core/src/audio.rs:69
let samples = self.samples.clone();  // Clone the Arc
```

This `Arc` is then shared with the audio callback:

```rust
// whis-core/src/audio.rs:107-111
move |data: &[T], _: &cpal::InputCallbackInfo| {
    let mut samples = samples.lock().unwrap();  // Lock the mutex
    for &sample in data {
        samples.push(cpal::Sample::from_sample(sample));
    }
}
```

The callback runs on a separate thread, but it safely mutates the shared `Vec` through the `Mutex`.

## Summary

**Key Takeaways:**

1. **`Box<T>`** - Heap allocation, single ownership
2. **`Rc<T>`** - Reference counting, single-threaded
3. **`Arc<T>`** - Atomic reference counting, thread-safe
4. **`Arc<Mutex<T>>`** - Shared mutable state across threads
5. **`Arc<dyn Trait>`** - Shared trait objects for polymorphism
6. **Clone `Arc` is cheap** - Only increments a counter

**Whis Patterns:**

- `Arc<Mutex<Vec<f32>>>` - Audio sample buffer
- `Arc<dyn TranscriptionBackend>` - Provider registry
- `Arc::clone()` when spawning tasks
- Lock guards in small scopes

## Exercise

Implement a thread-safe counter using `Arc<Mutex<T>>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = // TODO: Create Arc<Mutex<i32>> starting at 0
    let mut handles = vec![];
    
    for _ in 0..10 {
        let counter = // TODO: Clone the Arc
        let handle = thread::spawn(move || {
            // TODO: Lock the mutex and increment the counter
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Result: {}", // TODO: Print the final counter value);
}
```

**Questions:**
1. Why do we need both `Arc` AND `Mutex`?
2. What happens if you forget to clone the `Arc` before moving into the thread?
3. How does the lock guard automatically release the lock?

---

Next: [Chapter 3: Traits & Generics](./ch03-traits.md)
