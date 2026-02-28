# KnotenCore

**A high-performance, AI-native compiler runtime that executes structured JSON Abstract Syntax Trees directly — powered by Rust.**

[![Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Why KnotenCore?

| Feature | Description |
|---|---|
| **JSON-AST Language** | Programs are pure JSON — no text parsing, no ambiguity. Perfect for AI code generation. |
| **AI-Native Design** | LLMs generate valid KnotenCore programs directly as structured data. Zero syntax errors. |
| **Static Type Inference** | Types are inferred and enforced before execution. Catch bugs at compile time. |
| **Automated Rust FFI** | Feed any `.rs` file to the ingestor — it generates typed KnotenCore bindings automatically. |
| **Struct Marshalling** | Pass complex objects across the FFI boundary. Rust structs ↔ KnotenCore Objects. |
| **AST Optimizer** | Constant folding and dead code elimination reduce your AST before it runs. |
| **WGPU Graphics** | Built-in 3D rendering pipeline with voxel engine, shaders, and real-time physics. |
| **Audio Engine** | Native audio synthesis with multi-voice waveform generation (Sine, Square, Saw, Tri, Noise). |

---

## Quickstart

```bash
# Clone
git clone https://github.com/holgerbaer-bl/knoten_core.git
cd knoten_core

# Run the v0.1.0 Showcase (demonstrates ALL features)
cargo run --bin run_knc examples/core/showcase_v1.nod
```

### Expected Output

```
========================================
  KnotenCore v0.1.0-alpha  —  Showcase
========================================

[1] Variables & Math
    Circle circumference (r=5): 31.4159

[2] Control Flow (If/Else)
    Result: EXCELLENT

[3] Arrays & Loops
    Sum of [10,20,30,40,50]: 150

[4] Functions
    factorial(10): 3628800

[5] Objects (Structs / Dictionaries)
    Player: AetherBot
    Level:  42

[6] Automated Rust FFI (ExternCall)
    normalize(3,4,0).x = 0.6
    normalize(3,4,0).y = 0.8
    hash('KnotenCore'): 5613

[7] Constant Folding (Optimizer)
    10 * 5 + 3 → folded to 53 at compile-time
```

---

## How It Works

KnotenCore programs are JSON files containing an Abstract Syntax Tree. Here's "Hello World":

```json
{
  "Print": { "StringLiteral": "Hello, World!" }
}
```

A more complex example — calling a **native Rust function** from KnotenCore:

```json
{
  "Block": [
    { "Import": "examples/core/test_lib.nod" },
    { "Assign": ["v", { "Call": ["Vector3", [
        { "FloatLiteral": 3.0 },
        { "FloatLiteral": 4.0 },
        { "FloatLiteral": 0.0 }
    ]]}]},
    { "Assign": ["n", { "Call": ["normalize_vector", [{ "Identifier": "v" }]]}]},
    { "Print": { "PropertyGet": [{ "Identifier": "n" }, "x"] }}
  ]
}
```

This constructs a `Vector3` object, passes it across the FFI bridge to a Rust `normalize_vector` function, and prints the result (`0.6`).

---

## Optimizer: Constant Folding

Before execution, KnotenCore's optimizer simplifies your AST:

```
Before:  { "Add": [{ "Mul": [{ "IntLiteral": 10 }, { "IntLiteral": 5 }] }, { "IntLiteral": 3 }] }
After:   { "IntLiteral": 53 }

AST reduced from 5 nodes → 1 node.
```

Dead code (e.g. `While(false, ...)`) is eliminated entirely.

---

## Automated Rust FFI

Convert any Rust library to KnotenCore bindings in one command:

```bash
cargo run --bin rust_ingest src/test_lib.rs
# → Generates examples/core/test_lib.nod with typed ExternCall wrappers
```

The ingestor parses `pub fn` and `pub struct` definitions, generating:
- **ExternCall wrappers** for functions
- **Constructor functions** for structs (returning `ObjectLiteral`)

The runtime bridge validates all struct fields at the FFI boundary with clean `[FFI Error]` messages.

---

## Repository Structure

```
src/
├── ast.rs              # AST node definitions & Type enum
├── executor.rs         # Runtime evaluation engine
├── optimizer.rs        # Constant folding, dead code elimination, type inference
├── validator.rs        # AST structural validation
├── lib.rs              # Crate exports
├── bin/
│   ├── run_knc.rs      # Main executable
│   └── rust_ingest.rs  # Rust → KnotenCore FFI generator
├── natives/
│   ├── math.rs         # Math native module
│   ├── io.rs           # I/O native module
│   └── bridge.rs       # FFI struct marshalling bridge
└── test_lib.rs         # Mock external Rust library

examples/
├── core/               # Language feature demos
│   ├── showcase_v1.nod # ← The Ultimate Demo
│   └── ...
└── voxel/              # 3D Voxel Engine showcase

docs/
├── KNOTEN_SPEC.md      # Language specification
├── RUST_INGEST.md      # FFI automation docs
├── STDLIB.md           # Standard library reference
└── AUDIT.md            # Optimization benchmarks

tests/                  # Rust integration tests
stdlib/                 # KnotenCore standard library modules
assets/                 # Textures, shaders, fonts
```

---

## Prerequisites

- [Rust](https://www.rust-lang.org/) (Latest Stable)
- A GPU compatible with Vulkan, Metal, or DX12 (for graphics features)

---

**Designed for Machine Intelligence. Powered by Rust.**
