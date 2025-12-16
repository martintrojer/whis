# Chapter 4: Error Handling

Rust's error handling is explicit—no exceptions, no hidden control flow. Understanding `Result<T, E>` and error propagation is essential for reading Whis code.

## The `Result<T, E>` Type

Rust uses `Result` for operations that can fail:

```rust
enum Result<T, E> {
    Ok(T),   // Success: contains value of type T
    Err(E),  // Failure: contains error of type E
}
```

### Basic Usage

```rust
use std::fs::File;

fn open_file() -> Result<File, std::io::Error> {
    File::open("hello.txt")  // Returns Result
}

fn main() {
    match open_file() {
        Ok(file) => println!("File opened: {:?}", file),
        Err(error) => println!("Error opening file: {}", error),
    }
}
```

## The `?` Operator: Error Propagation

The `?` operator propagates errors up the call stack:

```rust
use std::fs::File;
use std::io::Read;

fn read_username() -> Result<String, std::io::Error> {
    let mut file = File::open("username.txt")?;  // Propagates error if open fails
    let mut username = String::new();
    file.read_to_string(&mut username)?;         // Propagates error if read fails
    Ok(username)
}
```

### What `?` Does

```rust
let file = File::open("file.txt")?;

// Is syntactic sugar for:
let file = match File::open("file.txt") {
    Ok(file) => file,
    Err(e) => return Err(e.into()),  // Convert error and return early
};
```

**Key points:**
1. Returns early on `Err`
2. Automatically converts error types with `.into()`
3. Only works in functions returning `Result` or `Option`

### Chaining with `?`

```rust
fn process() -> Result<(), std::io::Error> {
    let content = std::fs::read_to_string("input.txt")?;
    let processed = content.trim();
    std::fs::write("output.txt", processed)?;
    Ok(())
}
```

Clean and readable—no nested `match` statements!

## `anyhow`: Simplified Error Handling

Whis uses the `anyhow` crate for ergonomic error handling:

```toml
[dependencies]
anyhow = "1.0"
```

### `anyhow::Result<T>`

Instead of specifying an error type, use `anyhow::Result`:

```rust
use anyhow::Result;

fn do_something() -> Result<String> {  // anyhow::Result<String>
    let file = std::fs::read_to_string("file.txt")?;
    Ok(file)
}
```

`anyhow::Result<T>` is shorthand for `Result<T, anyhow::Error>`.

### `anyhow::Error` - Type-Erased Errors

`anyhow::Error` can hold **any error type**:

```rust
use anyhow::Result;
use std::fs::File;
use std::io::Read;

fn read_config() -> Result<String> {
    let mut file = File::open("config.toml")?;  // std::io::Error
    let mut config = String::new();
    file.read_to_string(&mut config)?;          // std::io::Error
    
    let parsed: toml::Value = toml::from_str(&config)?;  // toml::de::Error
    
    Ok(parsed.to_string())
}
```

All different error types (`io::Error`, `toml::de::Error`) are automatically converted to `anyhow::Error`.

### How Error Conversion Works

The `?` operator uses `From<E>` to convert errors:

```rust
// When you write:
let file = File::open("test.txt")?;

// The compiler generates:
let file = match File::open("test.txt") {
    Ok(f) => f,
    Err(e) => return Err(anyhow::Error::from(e)),  // Automatic conversion
};
```

## Adding Context with `.context()`

`anyhow` provides `.context()` to add error descriptions:

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<String> {
    let config = std::fs::read_to_string("config.toml")
        .context("Failed to read config.toml")?;
    
    Ok(config)
}
```

If the file read fails, the error message becomes:

```
Failed to read config.toml

Caused by:
    No such file or directory (os error 2)
```

### Context Chains

You can add multiple context layers:

```rust
use anyhow::{Context, Result};

fn parse_port(config: &str) -> Result<u16> {
    config.parse()
        .context("Invalid port number")
        .context("Failed to parse configuration")?
}
```

Error output:

```
Failed to parse configuration

Caused by:
    0: Invalid port number
    1: invalid digit found in string
```

### `.with_context()` for Lazy Evaluation

Use `.with_context()` with a closure for expensive context:

```rust
use anyhow::{Context, Result};

fn process(items: &[Item]) -> Result<()> {
    for item in items {
        do_something(item)
            .with_context(|| format!("Failed to process item: {:?}", item))?;
    }
    Ok(())
}
```

The closure only runs if there's an error.

## The `bail!()` Macro

`bail!()` returns an error immediately:

```rust
use anyhow::{bail, Result};

fn check_positive(x: i32) -> Result<()> {
    if x < 0 {
        bail!("x must be positive, got {}", x);
    }
    Ok(())
}
```

It's shorthand for:

```rust
return Err(anyhow::anyhow!("x must be positive, got {}", x));
```

### When to Use `bail!`

```rust
use anyhow::{bail, Result};

fn validate_audio(samples: &[f32]) -> Result<()> {
    if samples.is_empty() {
        bail!("No audio data recorded");
    }
    
    if samples.len() < 1000 {
        bail!("Recording too short: {} samples", samples.len());
    }
    
    Ok(())
}
```

This is cleaner than deeply nested `if` statements with `Err` returns.

## Error Handling in Whis

Let's see how Whis uses `anyhow`:

### Example 1: Audio Recording

```rust
// whis-core/src/audio.rs:132-134
if samples.is_empty() {
    anyhow::bail!("No audio data recorded");
}
```

Simple validation with `bail!`.

### Example 2: File Operations with Context

```rust
// whis-core/src/audio.rs:283
let mp3_data = std::fs::read(&mp3_path)
    .context("Failed to read converted MP3 file")?;
```

Adding context to I/O errors.

### Example 3: Provider Errors

```rust
// whis-core/src/provider/openai.rs (conceptual)
if !response.status().is_success() {
    let status = response.status();
    let error_text = response
        .text()
        .unwrap_or_else(|_| "Unknown error".to_string());
    anyhow::bail!("API error ({status}): {error_text}");
}
```

Constructing detailed error messages from API responses.

## Custom Errors vs `anyhow`

### When to Use Custom Error Types

For **libraries**, define custom error enums with `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration")]
    InvalidConfig,
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

Custom errors provide:
- Type-safe error matching
- Programmatic error handling
- Clear API contracts

### When to Use `anyhow`

For **applications** (like Whis), use `anyhow`:

- Simpler code
- Less boilerplate
- Focus on error messages, not types
- Errors propagate to the user, not other code

**Rule of thumb**:
- **Library code**: Use `thiserror` for typed errors
- **Application code**: Use `anyhow` for ergonomic errors

Whis is an application, so `anyhow` everywhere is appropriate.

## The `unwrap()` and `expect()` Methods

### `unwrap()` - Panic on Error

```rust
let file = File::open("test.txt").unwrap();  // Panics if Err
```

**When to use:**
- Quick prototyping
- Tests
- When error is genuinely impossible

**Avoid in production code** unless you have a good reason.

### `expect()` - Panic with Message

```rust
let file = File::open("test.txt")
    .expect("test.txt must exist for this program to work");
```

Better than `unwrap()` because it documents **why** the panic is acceptable.

### In Whis

You'll see `.unwrap()` in a few places:

```rust
// whis-core/src/audio.rs:70
samples.lock().unwrap().clear();
```

Why is `unwrap()` okay here?

- Mutex poisoning means a thread panicked while holding the lock
- This indicates a serious bug—we **want** the program to crash
- Recovering from a poisoned mutex rarely makes sense

## Error Handling Best Practices

### 1. Use `?` for Propagation

```rust
// Good
fn foo() -> Result<()> {
    let x = might_fail()?;
    Ok(())
}

// Bad
fn foo() -> Result<()> {
    match might_fail() {
        Ok(x) => { /* ... */ },
        Err(e) => return Err(e),
    }
    Ok(())
}
```

### 2. Add Context to Errors

```rust
// Good
let data = std::fs::read("config.toml")
    .context("Failed to read configuration file")?;

// Bad
let data = std::fs::read("config.toml")?;  // Generic error message
```

### 3. Use `bail!` for Custom Errors

```rust
// Good
if items.is_empty() {
    bail!("Cannot process empty list");
}

// Bad
if items.is_empty() {
    return Err(anyhow::anyhow!("Cannot process empty list"));
}
```

### 4. Avoid `unwrap()` in Library Code

```rust
// Good (library)
pub fn process(data: &[u8]) -> Result<Output> {
    let parsed = parse(data)?;
    Ok(Output { parsed })
}

// Bad (library)
pub fn process(data: &[u8]) -> Output {
    let parsed = parse(data).unwrap();  // Caller can't handle errors!
    Output { parsed }
}
```

## Summary

**Key Takeaways:**

1. **`Result<T, E>`** is Rust's way of handling errors explicitly
2. **`?` operator** propagates errors up the call stack
3. **`anyhow::Result<T>`** simplifies error types in applications
4. **`.context()`** adds descriptive error messages
5. **`bail!()`** returns custom errors immediately
6. **`unwrap()`/`expect()`** should be rare in production code

**Whis Patterns:**

- `anyhow::Result<T>` for all fallible functions
- `.context()` on I/O operations
- `bail!()` for validation errors
- `.unwrap()` only on mutexes (where poisoning = bug)

## Exercise

Refactor this code to use `anyhow`:

```rust
use std::fs;
use std::io;

fn read_and_parse() -> Result<i32, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("number.txt")?;
    let number: i32 = content.trim().parse()?;
    
    if number < 0 {
        return Err("Number must be positive".into());
    }
    
    Ok(number)
}
```

**Tasks:**
1. Replace `Box<dyn std::error::Error>` with `anyhow::Result`
2. Add `.context()` to the file read
3. Use `bail!()` for the validation error

**Questions:**
1. Why is `anyhow::Result` simpler than `Box<dyn Error>`?
2. How does `.context()` improve debugging?
3. When would you NOT want to use `anyhow`?

---

Next: [Chapter 5: Async Rust Fundamentals](./ch05-async.md)
