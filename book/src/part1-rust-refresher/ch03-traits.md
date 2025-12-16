# Chapter 3: Traits & Generics

Traits and generics are how Rust achieves polymorphism and code reuse. Understanding them is essential for grasping Whis's provider system and API design.

## Traits: Shared Behavior

A **trait** defines functionality a type must provide. Think of it as an interface or contract:

```rust
trait Summary {
    fn summarize(&self) -> String;
}

struct Article {
    title: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{}: {}", self.title, self.content)
    }
}

fn main() {
    let article = Article {
        title: String::from("Rust Traits"),
        content: String::from("Traits enable polymorphism"),
    };
    
    println!("{}", article.summarize());
}
```

### Default Implementations

Traits can provide default method implementations:

```rust
trait Summary {
    fn summarize_author(&self) -> String;
    
    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
    // summarize() uses the default implementation
}
```

### Supertraits

Traits can depend on other traits:

```rust
trait Display {
    fn display(&self) -> String;
}

trait Debug: Display {  // Debug requires Display
    fn debug(&self) -> String {
        format!("Debug: {}", self.display())
    }
}
```

Any type implementing `Debug` must also implement `Display`.

## Generics: Type Parameters

Generics let you write code that works with multiple types:

```rust
fn largest<T>(list: &[T]) -> &T 
where
    T: PartialOrd,
{
    let mut largest = &list[0];
    
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    
    largest
}

fn main() {
    let numbers = vec![34, 50, 25, 100];
    println!("Largest: {}", largest(&numbers));
    
    let chars = vec!['y', 'm', 'a', 'q'];
    println!("Largest: {}", largest(&chars));
}
```

### Generic Syntax

```rust
fn function_name<T>(param: T) -> T {
    // Function body
}

struct Point<T> {
    x: T,
    y: T,
}

enum Option<T> {
    Some(T),
    None,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}
```

## Trait Bounds

Trait bounds restrict which types can be used with generics:

### Basic Bounds

```rust
fn print_it<T: std::fmt::Display>(item: T) {
    println!("{}", item);
}
```

This says: "`T` can be any type that implements `Display`".

### Multiple Bounds

```rust
fn process<T: Clone + std::fmt::Display>(item: T) {
    let cloned = item.clone();
    println!("{}", cloned);
}
```

`T` must implement BOTH `Clone` AND `Display`.

### `where` Clauses

For complex bounds, use `where` for readability:

```rust
fn complex_function<T, U>(t: T, u: U) -> String
where
    T: std::fmt::Display + Clone,
    U: Clone + std::fmt::Debug,
{
    format!("{} {:?}", t, u)
}
```

### Bound Syntax Comparison

```rust
// Inline syntax
fn foo<T: Clone + Display>(t: T) { }

// Where clause (preferred for complex bounds)
fn foo<T>(t: T)
where
    T: Clone + Display,
{ }
```

## Associated Types

Associated types make traits more ergonomic by not requiring type parameters:

```rust
trait Iterator {
    type Item;  // Associated type
    
    fn next(&mut self) -> Option<Self::Item>;
}

struct Counter {
    count: u32,
}

impl Iterator for Counter {
    type Item = u32;  // Concrete type for Item
    
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        Some(self.count)
    }
}
```

### Associated Types vs Generic Parameters

```rust
// With generic parameter (verbose):
trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}

// Usage requires specifying type:
fn use_iterator<T, I: Iterator<T>>(iter: &mut I) { }

// With associated type (cleaner):
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Usage infers the type:
fn use_iterator<I: Iterator>(iter: &mut I) { }
```

**Rule of thumb**: Use associated types when there's a single obvious output type.

## Static Dispatch with `impl Trait`

`impl Trait` syntax is syntactic sugar for generics:

### In Parameters

```rust
// These are equivalent:
fn notify1(item: &impl Summary) {
    println!("{}", item.summarize());
}

fn notify2<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}
```

> **Use `impl Trait`** for simple cases. **Use generic `<T>`** when you need to refer to the type multiple times or add complex bounds.

### In Return Types

```rust
fn returns_summarizable() -> impl Summary {
    Article {
        title: String::from("Title"),
        content: String::from("Content"),
    }
}
```

This hides the concrete return type from callers. They only know it implements `Summary`.

**Limitation**: You can only return ONE concrete type:

```rust
// ERROR: Two different types!
fn returns_summarizable(switch: bool) -> impl Summary {
    if switch {
        Article { /* ... */ }  // Type 1
    } else {
        Tweet { /* ... */ }    // Type 2 - ERROR!
    }
}
```

For returning different types, use trait objects (next section).

## Dynamic Dispatch with Trait Objects

Trait objects enable **runtime polymorphism**:

```rust
trait Draw {
    fn draw(&self);
}

struct Circle { radius: f64 }
impl Draw for Circle {
    fn draw(&self) {
        println!("Drawing circle, radius: {}", self.radius);
    }
}

struct Rectangle { width: f64, height: f64 }
impl Draw for Rectangle {
    fn draw(&self) {
        println!("Drawing rectangle, {}x{}", self.width, self.height);
    }
}

fn main() {
    let shapes: Vec<Box<dyn Draw>> = vec![
        Box::new(Circle { radius: 5.0 }),
        Box::new(Rectangle { width: 3.0, height: 4.0 }),
    ];
    
    for shape in &shapes {
        shape.draw();  // Runtime polymorphism!
    }
}
```

### Trait Object Syntax

```rust
&dyn Trait      // Borrowed trait object
Box<dyn Trait>  // Owned trait object (heap allocated)
Arc<dyn Trait>  // Shared trait object (reference counted)
```

### How Trait Objects Work

Under the hood, trait objects use a **vtable** (virtual method table):

```
┌───────────────┐
│  Data Pointer │ ────> Actual object (Circle, Rectangle, etc.)
├───────────────┤
│ VTable Pointer│ ────> Function pointers (draw, etc.)
└───────────────┘
```

Each method call goes through the vtable, adding a small runtime cost.

## Static vs Dynamic Dispatch

| Feature | Static (`impl Trait` / generics) | Dynamic (`dyn Trait`) |
|---------|----------------------------------|----------------------|
| **Performance** | Fast (inlined, monomorphized) | Slower (vtable lookup) |
| **Code size** | Larger (duplicate code per type) | Smaller (one copy) |
| **Flexibility** | Compile-time only | Runtime polymorphism |
| **Return type** | Must be single concrete type | Can be multiple types |

### Static Dispatch Example

```rust
fn process<T: Summary>(item: T) {
    println!("{}", item.summarize());
}

// Compiler generates:
// fn process_Article(item: Article) { ... }
// fn process_Tweet(item: Tweet) { ... }
```

Each type gets its own compiled function (**monomorphization**).

### Dynamic Dispatch Example

```rust
fn process(item: &dyn Summary) {
    println!("{}", item.summarize());  // Vtable lookup at runtime
}

// One function works with all types!
```

### When to Use Each

**Use static dispatch (generics)** when:
- Performance is critical
- Types are known at compile time
- Code size isn't a concern

**Use dynamic dispatch (trait objects)** when:
- You need heterogeneous collections (`Vec<Box<dyn Trait>>`)
- Return types vary at runtime
- Plugin/extension systems

## Object Safety

Not all traits can be trait objects. A trait is **object-safe** if:

1. **No generic methods** (can't be in vtable)
2. **No `Self: Sized` bound** (trait objects aren't sized)
3. **Return type isn't `Self`** (unknown concrete type)

```rust
trait Bad {
    fn generic<T>(&self, x: T);  // ERROR: Generic method
    fn returns_self(&self) -> Self;  // ERROR: Returns Self
}

// Cannot create: Box<dyn Bad>
```

**Most traits in Whis are object-safe** because they're designed for trait objects.

## Whis's Provider Trait

Let's see how Whis uses traits for its transcription provider system:

```rust
// whis-core/src/provider/mod.rs:151-176
#[async_trait]
pub trait TranscriptionBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    
    fn transcribe_sync(
        &self,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
    
    async fn transcribe_async(
        &self,
        client: &reqwest::Client,
        api_key: &str,
        request: TranscriptionRequest,
    ) -> Result<TranscriptionResult>;
}
```

Breaking this down:

### 1. `#[async_trait]` Macro

Enables async methods in traits (we'll cover this in Chapter 5).

### 2. Supertraits: `Send + Sync`

```rust
pub trait TranscriptionBackend: Send + Sync {
    // ...
}
```

This means: "Any type implementing `TranscriptionBackend` must also be `Send + Sync`".

- `Send` - Safe to move between threads
- `Sync` - Safe to share references between threads

**Why?** Providers are used in async tasks that may run on different threads.

### 3. Associated Lifetime: `&'static str`

```rust
fn name(&self) -> &'static str;
```

Provider names live for the entire program (string literals like `"openai"`).

### 4. Sync and Async Methods

The trait provides both:
- `transcribe_sync()` - Blocking call for simple cases
- `transcribe_async()` - Async call for parallel processing

This flexibility lets callers choose the appropriate method.

## Provider Registry with Trait Objects

The provider registry uses `Arc<dyn TranscriptionBackend>`:

```rust
// whis-core/src/provider/mod.rs:179-181
pub struct ProviderRegistry {
    providers: HashMap<&'static str, Arc<dyn TranscriptionBackend>>,
}
```

### Why `dyn`?

Without trait objects, we'd need an enum:

```rust
// Alternative: Enum dispatch (not extensible)
enum Provider {
    OpenAI(OpenAIProvider),
    Groq(GroqProvider),
    Deepgram(DeepgramProvider),
    // Can't add new providers without modifying this enum!
}
```

With `dyn`, we can register any type implementing the trait:

```rust
providers.insert("openai", Arc::new(OpenAIProvider));
providers.insert("groq", Arc::new(GroqProvider));
// Extensible: Add new providers without changing the registry
```

### Why `Arc`?

Multiple parts of the code need access to providers:
- CLI command execution
- Desktop app recording
- Mobile app recording

`Arc<dyn Trait>` allows cheap cloning of the pointer while sharing the provider implementation.

## Deriving Traits

Common traits can be automatically implemented:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}
```

**Derivable traits:**
- `Debug` - `{:?}` formatting
- `Clone` - `.clone()` method
- `Copy` - Bitwise copy
- `PartialEq`, `Eq` - Equality comparison
- `PartialOrd`, `Ord` - Ordering
- `Hash` - Hashing for collections

## Summary

**Key Takeaways:**

1. **Traits** define shared behavior (like interfaces)
2. **Generics** enable code reuse with type parameters
3. **Trait bounds** constrain which types work with generics
4. **Static dispatch** (`impl Trait`, generics) - Fast, compile-time
5. **Dynamic dispatch** (`dyn Trait`) - Flexible, runtime polymorphism
6. **Associated types** avoid verbose type parameters
7. **Object safety** determines if a trait can be a trait object

**Whis Patterns:**

- `TranscriptionBackend` trait - Defines provider interface
- `Arc<dyn TranscriptionBackend>` - Shared trait objects
- `Send + Sync` bounds - Thread-safe trait objects
- `&'static str` - Lifetime for provider names

## Exercise

Implement a simple trait and use it with both static and dynamic dispatch:

```rust
trait Speak {
    fn speak(&self) -> String;
}

struct Dog { name: String }
struct Cat { name: String }

// TODO: Implement Speak for Dog and Cat

fn static_dispatch<T: Speak>(animal: &T) {
    println!("{}", animal.speak());
}

fn dynamic_dispatch(animal: &dyn Speak) {
    println!("{}", animal.speak());
}

fn main() {
    let dog = Dog { name: String::from("Buddy") };
    let cat = Cat { name: String::from("Whiskers") };
    
    // TODO: Call static_dispatch with dog
    // TODO: Call dynamic_dispatch with cat
    
    // TODO: Create a Vec<Box<dyn Speak>> and iterate over it
}
```

**Questions:**
1. What's the difference in how the compiler handles `static_dispatch` vs `dynamic_dispatch`?
2. Why can you put `Dog` and `Cat` in the same `Vec<Box<dyn Speak>>`?
3. What happens if you try to add a generic method to `Speak`?

---

Next: [Chapter 4: Error Handling](./ch04-errors.md)
