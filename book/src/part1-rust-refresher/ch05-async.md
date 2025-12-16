# Chapter 5: Async Rust Fundamentals

Asynchronous programming in Rust enables concurrent operations without threads. Whis uses async for parallel transcription, making this chapter essential.

## What is Async/Await?

**Synchronous code** blocks until operations complete:

```rust
fn fetch_data() -> String {
    // Blocks the thread while waiting for network response
    reqwest::blocking::get("https://api.example.com")
        .unwrap()
        .text()
        .unwrap()
}
```

**Asynchronous code** allows other work while waiting:

```rust
async fn fetch_data() -> String {
    // Yields control while waiting, other tasks can run
    reqwest::get("https://api.example.com")
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}
```

### The Async Model

```
Synchronous:          Asynchronous:
┌──────────┐          ┌──────────┐
│ Task 1   │          │ Task 1   │─ await ─┐
│ (waits)  │          ├──────────┤         │
│          │          │ Task 2   │         │ (CPU does other work)
│          │          ├──────────┤         │
│ (done)   │          │ Task 3   │         │
└──────────┘          ├──────────┤         │
                      │ Task 1   │←────────┘
                      │ (resumes)│
                      └──────────┘
```

Async code **doesn't block** while waiting for I/O.

## Futures: The Building Blocks

An `async fn` returns a **Future**:

```rust
async fn hello() -> String {
    String::from("Hello")
}

fn main() {
    let future = hello();  // Future<Output = String>
    // Nothing has happened yet! Futures are lazy.
}
```

### Futures Are Lazy

```rust
async fn print_message() {
    println!("This won't print!");
}

fn main() {
    let fut = print_message();  // Creates future, but doesn't run it
    // Future is dropped, never executed
}
```

To run a future, you must **await** or **spawn** it.

### The `.await` Keyword

`.await` suspends the current async function until the future completes:

```rust
async fn fetch_and_process() {
    let data = fetch_data().await;  // Suspend here until data arrives
    process(data);
}
```

**Important**: `.await` only works inside `async` functions!

```rust
fn synchronous() {
    let data = fetch_data().await;  // ERROR: Not in async context
}
```

## The Tokio Runtime

Futures need a **runtime** to execute them. Whis uses **Tokio**:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

### Running Async Code

```rust
#[tokio::main]
async fn main() {
    let result = hello().await;
    println!("{}", result);
}
```

The `#[tokio::main]` macro transforms `main` into:

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let result = hello().await;
        println!("{}", result);
    });
}
```

### Manual Runtime Creation

For more control:

```rust
fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        // Your async code here
    });
}
```

## Spawning Tasks with `tokio::spawn`

`tokio::spawn` runs futures concurrently on the runtime:

```rust
use tokio::task;

#[tokio::main]
async fn main() {
    let handle1 = task::spawn(async {
        println!("Task 1");
        42
    });
    
    let handle2 = task::spawn(async {
        println!("Task 2");
        100
    });
    
    let result1 = handle1.await.unwrap();
    let result2 = handle2.await.unwrap();
    
    println!("Results: {} and {}", result1, result2);
}
```

### `spawn` vs `.await`

```rust
// Sequential: One after another
let a = task1().await;
let b = task2().await;

// Concurrent: Both run at the same time
let handle1 = tokio::spawn(task1());
let handle2 = tokio::spawn(task2());
let a = handle1.await.unwrap();
let b = handle2.await.unwrap();
```

### JoinHandle

`tokio::spawn` returns a `JoinHandle`:

```rust
let handle: JoinHandle<i32> = tokio::spawn(async {
    42
});

let result: Result<i32, JoinError> = handle.await;
```

- **`JoinHandle<T>`** - Handle to the spawned task
- **`.await`** - Wait for task completion
- **Returns `Result<T, JoinError>`** - `Err` if task panicked

## Moving Data into Async Tasks

Tasks need `move` closures to own their data:

```rust
use std::sync::Arc;

async fn process_data() {
    let data = Arc::new(vec![1, 2, 3]);
    
    let handle = tokio::spawn(async move {
        // Takes ownership of `data`
        println!("{:?}", data);
    });
    
    // `data` moved, can't use it here
    handle.await.unwrap();
}
```

### Sharing with `Arc`

To share data between tasks:

```rust
use std::sync::Arc;

async fn concurrent_processing() {
    let data = Arc::new(vec![1, 2, 3]);
    
    let data1 = Arc::clone(&data);
    let handle1 = tokio::spawn(async move {
        println!("Task 1: {:?}", data1);
    });
    
    let data2 = Arc::clone(&data);
    let handle2 = tokio::spawn(async move {
        println!("Task 2: {:?}", data2);
    });
    
    handle1.await.unwrap();
    handle2.await.unwrap();
}
```

## `spawn_blocking` for CPU-Bound Work

`tokio::spawn` is for async I/O. For CPU-intensive work, use `spawn_blocking`:

```rust
use tokio::task;

async fn process() {
    let result = task::spawn_blocking(|| {
        // Blocking CPU work happens on a thread pool
        expensive_computation()
    }).await.unwrap();
    
    println!("Result: {}", result);
}

fn expensive_computation() -> i32 {
    // Simulate heavy computation
    (0..1_000_000).sum()
}
```

### When to Use Each

| Use Case | Function | Thread Pool |
|----------|----------|-------------|
| Async I/O (network, files) | `tokio::spawn` | Async runtime |
| CPU-bound work | `spawn_blocking` | Dedicated thread pool |
| Sync blocking I/O | `spawn_blocking` | Dedicated thread pool |

**In Whis**:

```rust
// whis-desktop/src/commands.rs (conceptual)
let output = tokio::task::spawn_blocking(move || {
    // MP3 encoding is CPU-bound
    recording.finalize()
}).await??;
```

## Parallel Transcription in Whis

Let's see how Whis uses async for parallel API requests:

```rust
// whis-core/src/transcribe.rs:52-122
pub async fn parallel_transcribe(
    provider: &TranscriptionProvider,
    api_key: &str,
    language: Option<&str>,
    chunks: Vec<AudioChunk>,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<String> {
    // Create shared HTTP client
    let client = Arc::new(reqwest::Client::new());
    let api_key = Arc::new(api_key.to_string());
    let provider_impl = registry().get_by_kind(provider);
    
    let mut handles = Vec::new();
    
    // Spawn ALL tasks immediately
    for chunk in chunks {
        let client = Arc::clone(&client);
        let api_key = Arc::clone(&api_key);
        let provider_impl = provider_impl.clone();
        
        let handle = tokio::spawn(async move {
            provider_impl.transcribe_async(&client, &api_key, request).await
        });
        
        handles.push(handle);
    }
    
    // Await all results
    for handle in handles {
        results.push(handle.await??);
    }
    
    Ok(merge_transcriptions(results))
}
```

### Breaking It Down

1. **Shared resources** - `Arc` for `client`, `api_key`, `provider_impl`
2. **Spawn all tasks** - Don't wait between spawns for maximum parallelism
3. **Collect handles** - Store `JoinHandle`s in a Vec
4. **Await all** - Wait for all tasks to complete
5. **Merge results** - Combine transcriptions

### Why This Pattern?

```rust
// BAD: Sequential (slow)
for chunk in chunks {
    let result = transcribe(chunk).await;  // Wait for each
    results.push(result);
}

// GOOD: Parallel (fast)
let mut handles = Vec::new();
for chunk in chunks {
    handles.push(tokio::spawn(transcribe(chunk)));  // Start all
}
for handle in handles {
    results.push(handle.await?);  // Wait for all
}
```

The second approach runs all transcriptions concurrently!

## Rate Limiting with Semaphores

Whis limits concurrent API requests with `tokio::sync::Semaphore`:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

const MAX_CONCURRENT: usize = 3;

async fn parallel_with_limit() {
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let permit = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _permit = permit.acquire().await.unwrap();  // Wait for slot
            println!("Task {} running", i);
            // Only 3 tasks run at once
            do_work().await;
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### In Whis's Parallel Transcription

```rust
// whis-core/src/transcribe.rs:67-68
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));

// Inside each task:
let _permit = semaphore.acquire_owned().await?;  // Wait for permit
```

This ensures no more than 3 API requests run simultaneously, preventing rate limit errors.

## The `async_trait` Crate

Rust doesn't natively support async trait methods. The `async_trait` crate fixes this:

```rust
use async_trait::async_trait;

#[async_trait]
trait Transcriber {
    async fn transcribe(&self, audio: &[u8]) -> Result<String>;
}
```

### What It Does

The macro transforms:

```rust
#[async_trait]
trait Transcriber {
    async fn transcribe(&self, audio: &[u8]) -> Result<String>;
}
```

Into:

```rust
trait Transcriber {
    fn transcribe<'life0, 'async_trait>(
        &'life0 self,
        audio: &'life0 [u8],
    ) -> Pin<Box<dyn Future<Output = Result<String>> + Send + 'async_trait>>;
}
```

You don't need to understand the details—just know it enables async in traits.

### In Whis

```rust
// whis-core/src/provider/mod.rs:154-155
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    // ...
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}
```

Without `#[async_trait]`, this wouldn't compile.

## Common Async Patterns

### Pattern 1: Parallel Operations

```rust
let (result1, result2, result3) = tokio::join!(
    fetch_data1(),
    fetch_data2(),
    fetch_data3(),
);
```

`tokio::join!` waits for all futures concurrently.

### Pattern 2: Race Conditions

```rust
let result = tokio::select! {
    r1 = fetch_fast() => r1,
    r2 = fetch_slow() => r2,
};
```

`tokio::select!` returns the first future that completes.

### Pattern 3: Timeout

```rust
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(5), fetch_data()).await;

match result {
    Ok(data) => println!("Got data: {:?}", data),
    Err(_) => println!("Timeout!"),
}
```

### Pattern 4: Channels for Communication

```rust
use tokio::sync::mpsc;

async fn producer_consumer() {
    let (tx, mut rx) = mpsc::channel(32);
    
    // Producer
    tokio::spawn(async move {
        tx.send(42).await.unwrap();
    });
    
    // Consumer
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

## Summary

**Key Takeaways:**

1. **Async functions** return Futures (lazy until awaited)
2. **`.await`** suspends execution until a Future completes
3. **Tokio runtime** executes async code
4. **`tokio::spawn`** runs tasks concurrently
5. **`spawn_blocking`** for CPU-bound work
6. **`Arc`** shares data across tasks
7. **Semaphores** limit concurrency
8. **`#[async_trait]`** enables async in traits

**Whis Patterns:**

- Parallel transcription with `tokio::spawn`
- Rate limiting with `Semaphore`
- `Arc<Client>` for shared HTTP client
- `spawn_blocking` for audio encoding
- `#[async_trait]` on `TranscriptionBackend`

## Exercise

Implement parallel API calls with rate limiting:

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

async fn fetch_url(id: usize) -> Result<String, reqwest::Error> {
    let url = format!("https://api.example.com/data/{}", id);
    reqwest::get(&url).await?.text().await
}

#[tokio::main]
async fn main() {
    let semaphore = Arc::new(Semaphore::new(3));  // Max 3 concurrent
    let mut handles = Vec::new();
    
    for i in 0..10 {
        // TODO: Clone semaphore
        let handle = tokio::spawn(async move {
            // TODO: Acquire permit
            // TODO: Call fetch_url(i)
            // TODO: Handle result
        });
        handles.push(handle);
    }
    
    // TODO: Await all handles
}
```

**Questions:**
1. Why do we spawn all tasks before awaiting any?
2. What happens if we don't use a semaphore?
3. How is this different from using threads?

---

**Congratulations!** You've completed Part I: Rust Refresher. 

Next: [Part II: Whis from 30,000 Feet](../part2-overview/README.md)
