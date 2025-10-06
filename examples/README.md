# dsl-ractor Examples

This directory contains examples demonstrating both async configurations supported by `dsl-ractor`.

## 🚀 Quick Start

### Default Mode (Native async-in-traits) - Recommended
```bash
cargo run --example dual_mode_example
cargo run --example counter_native_async  
cargo run --example counter
cargo run --example ping_pong
```

### With async-trait Support
```bash
cargo run --example dual_mode_example --features async-trait
cargo run --example counter_async_trait --features async-trait
```

## 📁 Example Files

### Core Examples
- **`dual_mode_example.rs`** - 🌟 **MAIN DEMO** - Shows identical code working in both modes
- **`counter.rs`** - Basic counter with increment/decrement operations
- **`ping_pong.rs`** - Self-messaging actor demonstration

### Mode-Specific Examples  
- **`counter_native_async.rs`** - Native async-in-traits version
- **`counter_async_trait.rs`** - async-trait crate version

## ⚙️ Configuration Examples

### 📦 Minimal Setup (Native async-in-traits)
```toml
[dependencies]
ractor = { version = "0.15.8", features = ["tokio_runtime"] }
dsl-ractor = "0.2.0"
tokio = { version = "1.47.1", features = ["full"] }
```

### 🔧 With async-trait Support
```toml
[dependencies]
ractor = { version = "0.15.8", features = ["tokio_runtime"] }
dsl-ractor = "0.2.0"
tokio = { version = "1.47.1", features = ["full"] }

[features]
async-trait = ["dsl-ractor/async-trait", "ractor/async-trait"]
```

## 🎯 Key Features Demonstrated

✅ **Identical API** - Same code works in both configurations  
✅ **Zero warnings** - Clean compilation in both modes  
✅ **State management** - Mutable state handling  
✅ **Message processing** - Pattern matching on messages  
✅ **Actor lifecycle** - Initialization with `actor_pre_start!`  
✅ **Error handling** - Result-based error propagation  

## 🧪 Testing Both Modes

The `dual_mode_example` is the best demonstration:

```bash
# Test native async (modern Rust)
cargo run --example dual_mode_example

# Test async-trait (compatibility)  
cargo run --example dual_mode_example --features async-trait
```

Both commands run **identical code** but use different async implementations under the hood!

## 🔍 What's Different?

**User Code:** Absolutely nothing! The macro handles everything.

**Generated Code:**
- **Native async:** Uses `impl Future<Output=...> + Send` return types
- **async-trait:** Uses `#[async_trait]` attribute on trait implementations

**Performance:** Native async has zero runtime overhead, async-trait has minimal overhead.

## 🎉 Recommendation

Use the **default mode** (native async-in-traits) unless you need compatibility with older Rust versions or specific async-trait requirements.