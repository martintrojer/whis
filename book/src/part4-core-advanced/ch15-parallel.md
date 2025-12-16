# Chapter 15: Parallel Transcription

When audio exceeds 20 MB, Whis splits it into chunks (Chapter 12). Transcribing them sequentially would be slow: 3 chunks × 2 minutes each = 6 minutes total. This chapter shows how Whis uses `tokio::spawn` and `Semaphore` to transcribe chunks in parallel while respecting API rate limits, then merges results by detecting overlapping text.

## The Problem: Large Files

A 30-minute recording at 44.1 kHz mono:
- Raw PCM: ~300 MB
- MP3 (128 kbps): ~28 MB

This exceeds most API limits (~25 MB). Solution: split into 5-minute chunks.

**Sequential transcription**:
```
Chunk 0: [====] 2 min
Chunk 1:      [====] 2 min
Chunk 2:           [====] 2 min
Total: 6 minutes
```

**Parallel transcription (3 concurrent)**:
```
Chunk 0: [====] 2 min
Chunk 1: [====] 2 min
Chunk 2: [====] 2 min
Total: 2 minutes
```

**3× speedup!**

## Constants

```rust
const MAX_CONCURRENT_REQUESTS: usize = 3;
const MAX_OVERLAP_WORDS: usize = 15;
```

**From `whis-core/src/transcribe.rs:16-18`**

**Why 3 concurrent requests?**
- Most APIs allow 3-5 concurrent requests before rate limiting
- More than 3 risks hitting rate limits
- 3 provides good parallelism without being aggressive

**Why 15 overlap words?**
- 2-second overlap ≈ 5-15 words (depending on speech speed)
- Searching more words wastes time
- Searching fewer might miss duplicates

## The `parallel_transcribe` Function

This is the heart of parallel transcription:

```rust
pub async fn parallel_transcribe(
    provider: &TranscriptionProvider,
    api_key: &str,
    language: Option<&str>,
    chunks: Vec<AudioChunk>,
    progress_callback: Option<Box<dyn Fn(usize, usize) + Send + Sync>>,
) -> Result<String>
```

**From `whis-core/src/transcribe.rs:52-58`**

**Parameters**:
- **`provider`**: Which API to use (OpenAI, Groq, etc.)
- **`api_key`**: API key for authentication
- **`language`**: Optional language hint (e.g., `"en"`)
- **`chunks`**: Audio chunks from Chapter 12's `finalize()`
- **`progress_callback`**: Optional GUI progress updater

## Step 1: Setup

```rust
let total_chunks = chunks.len();

// Create shared HTTP client with timeout
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS))
    .build()
    .context("Failed to create HTTP client")?;

// Semaphore to limit concurrent requests
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
let client = Arc::new(client);
let api_key = Arc::new(api_key.to_string());
let language = language.map(|s| Arc::new(s.to_string()));
let provider_impl = registry().get_by_kind(provider);
let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
let progress_callback = progress_callback.map(Arc::new);
```

**From `whis-core/src/transcribe.rs:59-74`**

### Shared HTTP Client

```rust
let client = Arc::new(reqwest::Client::new());
```

**Why share one client?**
- Connection pooling: Reuses HTTP connections
- Faster: Avoid TCP handshake for each request
- Lower overhead: One client instead of N clients

### Semaphore for Rate Limiting

```rust
let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_REQUESTS));
```

**`Semaphore`** (from `tokio::sync`):
- Limits concurrent access to a resource
- `new(3)` allows 3 concurrent permits
- `acquire()` blocks if all permits are taken
- Releasing permit allows waiting tasks to proceed

**Think of it as**:
```
Semaphore = [Permit1, Permit2, Permit3]

Task A acquires Permit1 → [-, Permit2, Permit3]
Task B acquires Permit2 → [-, -, Permit3]
Task C acquires Permit3 → [-, -, -]
Task D waits...

Task A finishes, releases → [Permit1, -, -]
Task D acquires Permit1 → [-, -, -]
```

### Wrapping in `Arc`

```rust
let api_key = Arc::new(api_key.to_string());
let language = language.map(|s| Arc::new(s.to_string()));
let provider_impl = registry().get_by_kind(provider); // Already Arc<dyn Trait>
```

**Why `Arc`?**  
Each spawned task needs access. `Arc` lets us share without copying the entire string N times.

**`provider_impl`** is already `Arc<dyn TranscriptionBackend>` from registry (Chapter 14), so just clone the Arc.

### Progress Tracking

```rust
let completed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
```

**`AtomicUsize`**: Thread-safe counter without mutex overhead.

## Step 2: Spawn All Tasks

```rust
let mut handles = Vec::with_capacity(total_chunks);

for chunk in chunks {
    let semaphore = semaphore.clone();
    let client = client.clone();
    let api_key = api_key.clone();
    let language = language.clone();
    let provider_impl = provider_impl.clone();
    let completed = completed.clone();
    let progress_callback = progress_callback.clone();

    let handle = tokio::spawn(async move {
        // Task body...
    });

    handles.push(handle);
}
```

**From `whis-core/src/transcribe.rs:76-122`**

**Key point**: **Spawn ALL tasks immediately**.

They don't all run at once—the semaphore controls that. But spawning them upfront lets tokio schedule them efficiently.

**Arc clones are cheap**:
```rust
let semaphore = semaphore.clone(); // Just increments ref count
```

No data is copied—just new pointers to the same `Semaphore`.

## Step 3: Task Body

Inside each spawned task:

```rust
let handle = tokio::spawn(async move {
    // 1. Acquire semaphore permit (blocks if >3 tasks running)
    let _permit = semaphore.acquire_owned().await?;

    let chunk_index = chunk.index;
    let has_leading_overlap = chunk.has_leading_overlap;

    // 2. Build request
    let request = TranscriptionRequest {
        audio_data: chunk.data,
        language: language.as_ref().map(|s| s.to_string()),
        filename: format!("audio_chunk_{chunk_index}.mp3"),
    };

    // 3. Call API
    let result = provider_impl
        .transcribe_async(&client, &api_key, request)
        .await;

    // 4. Handle result
    let transcription = match result {
        Ok(r) => ChunkTranscription {
            index: chunk_index,
            text: r.text,
            has_leading_overlap,
        },
        Err(e) => return Err(e),
    };

    // 5. Update progress
    let done = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
    if let Some(ref cb) = progress_callback {
        cb(done, total_chunks);
    }

    Ok(transcription)
});
```

**From `whis-core/src/transcribe.rs:88-119`**

### Semaphore Acquisition

```rust
let _permit = semaphore.acquire_owned().await?;
```

**`acquire_owned()`**: 
- Waits until a permit is available
- Returns `OwnedSemaphorePermit`
- Automatically releases permit when dropped

**Why `_permit` (with underscore)?**  
The variable isn't used—we just need it to exist. When it drops at end of scope, the permit is released.

**Example timeline**:
```
Time 0: Task 0, 1, 2 acquire permits → All 3 running
Time 1: Task 3, 4, 5 wait (no permits available)
Time 2: Task 0 finishes → Permit released → Task 3 starts
Time 3: Task 1 finishes → Task 4 starts
...
```

### Progress Callback

```rust
let done = completed.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
if let Some(ref cb) = progress_callback {
    cb(done, total_chunks);
}
```

**`fetch_add(1, ...)`**: Atomically increment and return old value.

**Example**:
```rust
// Initial: completed = 0
let old = completed.fetch_add(1, SeqCst); // old = 0, completed = 1
let done = old + 1;                        // done = 1

// Desktop GUI callback:
cb(1, 6); // "Chunk 1 of 6 complete"
```

**`SeqCst` (Sequentially Consistent)**:  
Strongest memory ordering. Ensures all threads see updates in the same order.

## Step 4: Collect Results

```rust
let mut results = Vec::with_capacity(total_chunks);
let mut errors = Vec::new();

for handle in handles {
    match handle.await {
        Ok(Ok(transcription)) => results.push(transcription),
        Ok(Err(e)) => errors.push(e),
        Err(e) => errors.push(anyhow::anyhow!("Task panicked: {e}")),
    }
}
```

**From `whis-core/src/transcribe.rs:125-134`**

**Double `Result` handling**:

1. **Outer `Result<T, JoinError>`**: Did the task panic?
   - `Ok(...)`: Task completed normally
   - `Err(JoinError)`: Task panicked

2. **Inner `Result<ChunkTranscription, anyhow::Error>`**: Did transcription succeed?
   - `Ok(transcription)`: Success
   - `Err(e)`: API error, network error, etc.

**Three outcomes**:
- `Ok(Ok(t))`: Success, add to results
- `Ok(Err(e))`: API failure, add to errors
- `Err(e)`: Panic, add to errors

### Error Aggregation

```rust
if !errors.is_empty() {
    let error_msgs: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
    anyhow::bail!(
        "Failed to transcribe {} of {} chunks:\n{}",
        errors.len(),
        total_chunks,
        error_msgs.join("\n")
    );
}
```

**From `whis-core/src/transcribe.rs:137-145`**

**Example error**:
```
Failed to transcribe 2 of 6 chunks:
Chunk 2: API error (429): Rate limit exceeded
Chunk 4: Network timeout after 300s
```

This gives the user detailed error information, not just "transcription failed."

## Step 5: Sort and Merge

```rust
// Sort by index to ensure correct order
results.sort_by_key(|r| r.index);

// Merge transcriptions
Ok(merge_transcriptions(results))
```

**From `whis-core/src/transcribe.rs:147-151`**

**Why sort?**  
Tasks complete in arbitrary order. Chunk 2 might finish before Chunk 0. Sorting ensures correct text order.

## Merging Transcriptions

The `merge_transcriptions` function combines chunks intelligently:

```rust
fn merge_transcriptions(transcriptions: Vec<ChunkTranscription>) -> String {
    if transcriptions.is_empty() {
        return String::new();
    }

    if transcriptions.len() == 1 {
        return transcriptions.into_iter().next().unwrap().text;
    }

    let mut merged = String::new();

    for (i, transcription) in transcriptions.into_iter().enumerate() {
        let text = transcription.text.trim();

        if i == 0 {
            // First chunk - use as-is
            merged.push_str(text);
        } else if transcription.has_leading_overlap {
            // This chunk has overlap - try to remove duplicate words
            let cleaned_text = remove_overlap(&merged, text);
            if !merged.ends_with(' ') && !cleaned_text.is_empty() {
                merged.push(' ');
            }
            merged.push_str(&cleaned_text);
        } else {
            // No overlap - just append with space
            if !merged.ends_with(' ') && !text.is_empty() {
                merged.push(' ');
            }
            merged.push_str(text);
        }
    }

    merged
}
```

**From `whis-core/src/transcribe.rs:155-190`**

**Three cases**:

### Case 1: First Chunk

```rust
if i == 0 {
    merged.push_str(text);
}
```

First chunk has no previous text—just use it verbatim.

### Case 2: Overlapping Chunk

```rust
else if transcription.has_leading_overlap {
    let cleaned_text = remove_overlap(&merged, text);
    if !merged.ends_with(' ') && !cleaned_text.is_empty() {
        merged.push(' ');
    }
    merged.push_str(&cleaned_text);
}
```

Recall from Chapter 12: chunks have 2-second overlap. The end of Chunk 0 contains the same words as the beginning of Chunk 1.

**Example**:
```
Chunk 0: "...and that's why I think the project is important."
Chunk 1: "think the project is important. Moving forward we need..."
                    ^^^^^^^^^^^^^^^^^^^^^^^
                    Duplicate words
```

`remove_overlap()` detects and removes the duplicate.

### Case 3: No Overlap

```rust
else {
    if !merged.ends_with(' ') && !text.is_empty() {
        merged.push(' ');
    }
    merged.push_str(text);
}
```

If no overlap flag, just append with space separator.

## Overlap Detection Algorithm

The `remove_overlap` function finds duplicate words:

```rust
fn remove_overlap(existing: &str, new_text: &str) -> String {
    let existing_words: Vec<&str> = existing.split_whitespace().collect();
    let new_words: Vec<&str> = new_text.split_whitespace().collect();

    if existing_words.is_empty() || new_words.is_empty() {
        return new_text.to_string();
    }

    // Look for overlap in the last N words of existing and first N words of new
    let search_end = existing_words.len().min(MAX_OVERLAP_WORDS);
    let search_new = new_words.len().min(MAX_OVERLAP_WORDS);

    // Find the longest matching overlap
    let mut best_overlap = 0;

    for overlap_len in 1..=search_end.min(search_new) {
        let end_slice = &existing_words[existing_words.len() - overlap_len..];
        let start_slice = &new_words[..overlap_len];

        // Case-insensitive comparison
        let matches = end_slice
            .iter()
            .zip(start_slice.iter())
            .all(|(a, b)| a.eq_ignore_ascii_case(b));

        if matches {
            best_overlap = overlap_len;
        }
    }

    if best_overlap > 0 {
        // Skip the overlapping words
        new_words[best_overlap..].join(" ")
    } else {
        new_text.to_string()
    }
}
```

**From `whis-core/src/transcribe.rs:193-230`**

### Step-by-Step Example

**Input**:
```rust
existing = "I think the project is";
new_text = "the project is important";
```

**Split into words**:
```rust
existing_words = ["I", "think", "the", "project", "is"];
new_words = ["the", "project", "is", "important"];
```

**Search for overlaps**:

| Overlap Length | End of Existing | Start of New | Match? |
|----------------|-----------------|--------------|--------|
| 1 | `["is"]` | `["the"]` | No |
| 2 | `["project", "is"]` | `["the", "project"]` | No |
| 3 | `["the", "project", "is"]` | `["the", "project", "is"]` | **Yes** |

**Best overlap**: 3 words

**Result**:
```rust
new_words[3..].join(" ") // Skip first 3 words
= ["important"].join(" ")
= "important"
```

**Merged**:
```
"I think the project is" + " " + "important"
= "I think the project is important"
```

Perfect! No duplicate words.

### Why Case-Insensitive?

```rust
.all(|(a, b)| a.eq_ignore_ascii_case(b))
```

Transcription APIs might capitalize differently:
- Chunk 0: `"...the Project is"`
- Chunk 1: `"the project is..."`

Case-insensitive matching ensures we still detect the overlap.

### Why Longest Match?

```rust
let mut best_overlap = 0;
for overlap_len in 1..=search_end.min(search_new) {
    if matches {
        best_overlap = overlap_len; // Keep updating
    }
}
```

**Example**:
- Overlap 1 word: `["is"]` matches
- Overlap 3 words: `["the", "project", "is"]` matches

We want the **longest match** (3 words), not the first match (1 word).

## Real-World Usage

### Desktop GUI with Progress

```rust
// whis-desktop/src/commands.rs (simplified)
#[tauri::command]
pub async fn transcribe_large_file(
    chunks: Vec<AudioChunk>,
    window: tauri::Window,
) -> Result<String, String> {
    let settings = load_settings();
    let api_key = settings.get_api_key().ok_or("No API key")?;

    let progress_cb = Box::new(move |done: usize, total: usize| {
        let _ = window.emit("transcription-progress", (done, total));
    });

    parallel_transcribe(
        &settings.provider,
        &api_key,
        settings.language.as_deref(),
        chunks,
        Some(progress_cb),
    )
    .await
    .map_err(|e| e.to_string())
}
```

**Frontend**:
```javascript
// Listen for progress events
listen('transcription-progress', (event) => {
    const [done, total] = event.payload;
    updateProgressBar(done / total * 100);
});
```

### CLI (No Progress)

```rust
// whis-cli/src/commands/transcribe.rs (simplified)
pub fn transcribe_large_file(chunks: Vec<AudioChunk>) -> Result<String> {
    let settings = Settings::load();
    let api_key = settings.get_api_key().context("No API key")?;

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        parallel_transcribe(
            &settings.provider,
            &api_key,
            settings.language.as_deref(),
            chunks,
            None, // No progress callback
        )
        .await
    })
}
```

CLI blocks on the async function using `block_on()`.

## Performance Analysis

### Speedup Calculation

For N chunks with T seconds transcription time each:

- **Sequential**: `N × T`
- **Parallel (3 concurrent)**: `⌈N / 3⌉ × T`

**Example** (6 chunks, 2 min each):
- Sequential: `6 × 2 = 12 minutes`
- Parallel: `⌈6 / 3⌉ × 2 = 4 minutes`

**Speedup**: 12 / 4 = **3×**

### Memory Usage

**Sequential**:
- 1 chunk in memory at a time: ~5 MB

**Parallel (3 concurrent)**:
- 3 chunks in memory: ~15 MB

Still very manageable.

### Network Bandwidth

**Upload**:
- 3 concurrent uploads × 5 MB = 15 MB simultaneously
- Requires decent internet connection (>5 Mbps upload)

**Download** (responses):
- JSON text responses are tiny (~5-50 KB)

## Error Handling Strategies

### Partial Failures

Currently, if any chunk fails, the entire transcription fails.

**Future improvement**: Retry failed chunks
```rust
for retry in 0..3 {
    match transcribe_chunk(chunk).await {
        Ok(result) => return Ok(result),
        Err(e) if retry < 2 => {
            tokio::time::sleep(Duration::from_secs(2)).await;
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### Rate Limiting

If API returns `429 (Too Many Requests)`, current code fails.

**Future improvement**: Exponential backoff
```rust
if status == 429 {
    let wait = 2u64.pow(retry);
    tokio::time::sleep(Duration::from_secs(wait)).await;
    retry_request().await?;
}
```

## Alternative Architectures

### Rayon for CPU-Bound Work

If you were encoding chunks (CPU-bound), use `rayon`:
```rust
use rayon::prelude::*;

let mp3_chunks: Vec<_> = audio_chunks
    .par_iter()
    .map(|chunk| encode_to_mp3(chunk))
    .collect();
```

But transcription is I/O-bound (network API calls), so tokio is correct choice.

### Async/Await vs Threads

**Why not threads?**
```rust
let handles: Vec<_> = chunks.into_iter()
    .map(|chunk| {
        std::thread::spawn(move || transcribe_sync(chunk))
    })
    .collect();
```

**Problems**:
- Each thread = 1-2 MB stack memory
- 6 chunks = 6-12 MB overhead
- Thread creation is slow (~1ms each)

**Tokio tasks**:
- ~2 KB per task
- 6 tasks = ~12 KB overhead
- Creation is instant

For I/O-bound work, async wins.

## Summary

**Key Takeaways:**

1. **Parallel speedup**: N chunks transcribed in ~⌈N/3⌉ time instead of N time
2. **`tokio::spawn`**: Spawn tasks for concurrent execution
3. **`Semaphore`**: Limit to 3 concurrent API requests (rate limiting)
4. **Arc clones**: Share client, API key, provider across tasks
5. **Overlap detection**: Remove duplicate words from chunked transcriptions
6. **Error aggregation**: Collect all errors, report them together

**Where This Matters in Whis:**

- Desktop GUI shows progress bar (`whis-desktop/src/commands.rs`)
- CLI transcribes large recordings (`whis-cli/src/commands/transcribe.rs`)
- Chunks created in `RecordingData::finalize()` (Chapter 12)
- Providers selected from registry (Chapter 14)

**Patterns Used:**

- **Semaphore pattern**: Control concurrent access to resource
- **Fan-out, fan-in**: Spawn many tasks, collect results
- **Atomic counters**: Thread-safe progress tracking
- **Overlap-based merging**: Handle chunked text intelligently

**Design Decisions:**

1. **Why 3 concurrent?** Balance between speed and rate limits
2. **Why spawn all upfront?** Let tokio schedule efficiently
3. **Why longest overlap match?** More accurate than first match
4. **Why case-insensitive?** APIs capitalize inconsistently

---

**Part IV Complete!**

You now understand Whis's advanced core systems:
- Audio encoding (FFmpeg, LAME, resampling)
- Provider system (trait objects, registry, polymorphism)
- Parallel transcription (tokio, semaphore, merging)

This completes the deep dive into `whis-core`. Next parts cover:
- Part V: CLI implementation
- Part VI: Tauri desktop app
- Part VII: Vue frontend
- Part VIII: Patterns and best practices
- Part IX: Build and deployment

---

Next: [Part V: The CLI Application](../part5-cli/README.md)
