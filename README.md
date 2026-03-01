# KnotenCore

KnotenCore is a Rust-based AOT compiler and JIT runtime for JSON-encoded Abstract Syntax Trees, featuring a deterministic ARC resource registry.

[![Rust](https://img.shields.io/badge/Language-Rust-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](./LICENSE)

---

## What it is

A KnotenCore program is a JSON file that describes an AST. The runtime either **interprets it immediately (JIT)** or **compiles it to a native binary (AOT)**. All OS resources — file handles, GPU contexts, timers, windows — are managed through a deterministic ARC registry that guarantees zero leaks without a garbage collector.

---

## How to Build (One-Click Compilation)

Convert any `.nod` script into a standalone native binary in a single command:

```bash
# JIT — interpret directly
cargo run --bin run_knc examples/final/standalone_demo.nod

# AOT — compile to named native binary
cargo run --release --bin run_knc -- build examples/final/standalone_demo.nod
# → produces standalone_demo.exe (or standalone_demo on Linux/macOS)
# → runs anywhere, no project folder required
```

The `build` command:
1. Parses and constant-folds the JSON-AST
2. Transpiles to Rust source
3. Creates a temp Cargo project with `knoten_core` as a path dependency
4. Compiles with `lto=fat`, `opt-level=3`, stripped symbols
5. Copies the named binary to the current directory

---

## Capabilities

| Feature | Description |
|---|---|
| **JSON-AST Language** | Programs are structured JSON — no text parser, no ambiguity |
| **JIT + AOT** | Interpret immediately or compile to a standalone native binary via `build` |
| **ARC Resource Registry** | File handles, GPU contexts, timers, windows — all zero-leak ARC managed |
| **Chronos Engine** | `registry_now()` / `registry_elapsed_ms()` — nanosecond-precision timing handles |
| **Visual Cortex** | `registry_gpu_init()` initialises a real WGPU device; `registry_fill_color()` for rendering |
| **AOT Codegen** | Transpiler emits type-safe Rust with automatic `registry_release()` injected at scope exit |
| **AST Optimiser** | Constant folding + dead code elimination before execution or compilation |

---

## Development Milestones

| Sprint | Feature | Result |
|--------|---------|--------|
| **Sprint 38–39** | Transpiler Foundation & Control Flow | AST → valid Rust codegen |
| **Sprint 40** | Hybrid ARC Engine | Transpiler handles + scope-drop injection |
| **Sprint 40.5** | Memory Safety Audit | Zero leaks on shadowing/reassignment |
| **Sprint 41** | Native IO Bridge | `std::fs::File` as ARC-managed handles |
| **Sprint 42** | Repository Hardening | Branding, gitignore, professional README |
| **Sprint 43** | Chronos Engine | 1M-iteration benchmark: JIT=651ms, AOT=**0ms** (LLVM constant-folded) |
| **Sprint 44** | Visual Cortex | AMD Radeon RX 6700 XT (Vulkan) initialised from a 60-line JSON script |
| **Sprint 45** | Standalone Compiler | `run_knc build` → `standalone_demo.exe` (157KB, LTO release) |

---

## Example Program

```json
{
  "Block": [
    { "Assign": ["timer", { "NativeCall": ["registry_now", []] }] },
    { "Assign": ["f",     { "NativeCall": ["registry_file_create", [{ "StringLiteral": "out.txt" }]] }] },
    { "NativeCall": ["registry_file_write", [{ "Identifier": "f" }, { "StringLiteral": "Hello!" }]] },
    { "Assign": ["ms", { "NativeCall": ["registry_elapsed_ms", [{ "Identifier": "timer" }]] }] },
    { "Print": { "Identifier": "ms" } }
  ]
}
```

All handles (`timer`, `f`) are automatically released when their block exits — no `free()`, no destructor, no GC pause.

---

## Repository Structure

```
src/
├── bin/
│   ├── run_knc.rs      # CLI: JIT runner + AOT build pipeline
│   └── aot_bench.rs    # AOT benchmark binary
├── compiler/codegen.rs # JSON-AST → Rust source transpiler
├── executor.rs         # JIT evaluation engine (3700 lines)
├── optimizer.rs        # Constant folding & dead code elimination
├── validator.rs        # Structural AST validation
└── natives/
    ├── registry.rs     # ARC resource registry (Counter/Window/File/Timestamp/GpuContext)
    ├── bridge.rs       # FFI dispatch table
    ├── io.rs           # IO native module
    └── math.rs         # Math native module

examples/
├── bench/heavy_load.nod    # 1M-iteration Chronos benchmark
├── graphics/blue_window.nod # Visual Cortex GPU demo
└── final/standalone_demo.nod # One-click-build deployment test
```

---

## Prerequisites

- [Rust](https://www.rust-lang.org/) (Latest Stable)
- Vulkan/DX12 compatible GPU (for graphics features)
