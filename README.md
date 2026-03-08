# KnotenCore 🦀🤖
**The Agent-First Rust Engine.**

## 1. What is KnotenCore?
**KnotenCore** is a high-performance, **Agent-Native** execution engine built entirely in Rust. It compiles and evaluates UI logic, graphics, audio, and state transformations instantly without an intermediate browser layer. Designed not for human boilerplate, but as a deterministic powerhouse that AI Agents can compile to efficiently and autonomously.

### Why "KnotenCore"? (No, it's not a German techno genre)
Despite sounding like an aggressive underground music style from Berlin, the name actually makes perfect sense for our architecture:
- **Knoten** is the German word for **Node**. Since our Neural DSL is basically a massive, highly efficient graph of Abstract Syntax Tree (AST) *nodes*, we just went with the German translation because... well, it sounds about 20% more over-engineered.
- **Core** represents the blazingly fast, bare-metal Rust execution environment that deterministically chews through these nodes without mercy.

So welcome to KnotenCore: Hardcore Nodes. We promise it won't tangle your logic.

## 2. Modular Engine Architecture (Sprint 72)
To maintain long-term stability and reduce compilation times, the core engine has been modularized into specialized components:

- **`src/executor.rs`**: The backbone of the engine. Acts as a lightweight **Coordinator** and **State-Holder** (`ExecutionEngine`). It orchestrates data flow between all other modules.
- **`src/evaluator.rs`**: The brain. Handles **AST Parsing**, recursive evaluation, and pure logical/mathematical execution.
- **`src/renderer.rs`**: The eyes. Unified **WGPU** logic, shader management, Hardware-Instancing, and high-performance draw calls.
- **`src/window.rs`**: The skin. Manages the **winit Event-Loop**, application lifecycle, and hardware input (MouseGrab/Keyboard).
- **`src/async_bridge.rs`**: The nervous system. Handles non-blocking operations like `Fetch` and `Extract` via background worker threads.

## 3. Security & Sandboxing (Sprint 76)
KnotenCore is built for AI-driven execution, which requires strict security. Starting with Sprint 76, the runner enforce a "Deny-by-Default" policy for I/O:
- **`FS Read/Write`**: Disabled by default.
- **`Permissions`**: Must be explicitly granted via CLI flags:
  - `--allow-read`: Enables `FSRead` and `registry_read_file`.
  - `--allow-write`: Enables `FSWrite` and `registry_write_file`.

## 4. Unified Physics System (Sprint 77)
KnotenCore features a unified AABB (Axis-Aligned Bounding Box) physics engine that bridges the voxel world and generic 3D space:
- **`AABB Collision`**: Scripts can register custom physical barriers using `AddWorldAABB`.
- **`FPS Integration`**: The camera movement automatically respects these boundaries, allowing for complex level design beyond simple voxels.
- **`Performance`**: Collision checks are optimized to handle hundreds of active world-AABBs per frame.

## 5. Automatic Memory Management (ARC)
Unlike raw handle systems, KnotenCore utilizes a **Managed Handle Topology**. Native resources (Windows, Textures, Counters) are wrapped in a `NativeHandle` struct that implements the `Drop` trait. When a handle variable goes out of scope in the DSL, the engine automatically decrements the reference count and cleans up the resource in the registry.

## 3. Why it exists ("Agent First")
The current app development ecosystem is heavily burdened with human-centric boilerplate, fragmented tooling, and bloated artifact pipelines. KnotenCore eliminates all of this overhead. By providing a **deterministic, token-efficient runtime expressly built for AIs**, it shifts the paradigm from "AI writing React code for humans" to "AI writing Neural DSL code for a bare-metal Agent VM."
It enables AI agents to read clear diagnostic JSON logs, self-heal instantly upon failure, and deliver highly optimized graphical applications (under 5MB).

## 3. The Power in Action: KnotenCalculator Pro v2.2
Our flagship demo, the **KnotenCalculator Pro v2.2 with Kinetic History**, proves the capabilities of the engine. Featuring a natively scrolling kinetic history tape, variable data states, and complex UI layouts, the Calculator evaluates the DOM entirely within KnotenCore's hybrid VM infrastructure at roughly 60+ FPS – powered exclusively by Agent-generated logic.

## 4. Native 2D Graphics API (Sprint 67)
To bypass the overhead of traditional UI layout engines, **KnotenCore** provides a native 2D drawing layer that renders directly via the GPU. This is essential for high-performance applications like games or complex data visualizations.

### Native 2D Nodes
- **`Node::DrawRect { x, y, width, height, color }`**: Paints a filled rectangle directly to the framebuffer. Uses EGUI's `layer_painter` on the background layer for zero-layout overhead.
- **`Node::UIFixed { width, height, body }`**: Forces a fixed pixel dimension on its children, bypassing responsive flow.
- **`Node::UIFillParent`**: Dynamically expands to fill the entire available screen or parent container rect.

### Architecture: The Hybrid Game Engine
Instead of wrapping every pixel in a "widget" object, `DrawRect` targets the low-level **Painter API**. This allows KnotenCore to function as a hybrid engine: a fast UI library for tools, and a blazingly fast 2D game engine for interactive experiences.

## 5. The Neural DSL
KnotenCore eschews heavy JSON trees for an Ultra-Dense Neural Syntax (`.knoten`). Designed for maximum structural compression and token efficiency, the DSL gives AI models immediate and obvious control flow mechanics.

```rust
// An elegant Agent snippet in Neural DSL
win = UIWindow("main_nav", "Control Panel") -> {
    grid(2, "layout_grid") -> {
        btn1 = UIButton("Initialize System");
        btn2 = UIButton("Launch Diagnostics");
        
        if (btn1) -> {
            FSWrite("sys.log", "System initialized.");
        }
    }
}
```

## 6. Architecture: The Hybrid AST/Register VM
KnotenCore dynamically routes code to the single most performant executor path. High-level UI declarations remain an AST, while intensive logical/mathematical constraints bypass the tree evaluator and compile directly into flat **Opcodes** for the Register VM.

```mermaid
graph TD
    A[Neural DSL Source Code] -->|Parser| B(Abstract Syntax Tree)
    B --> C{Execution Router}
    
    C -->|Math / Pure Logic| D[VM Compiler]
    D -->|Opcodes| E((Register VM))
    E --> F[Execution Result]
    
    C -->|UI / Side Effects| G[Graph Executor]
    G --> H[egui / WGPU System]
    H --> F
    
    B -.->|JSON Diagnostic Log| I[AI Agent Self-Healing Loop]
```

### Supported Platforms
- Windows `x86_64`
- macOS `x86_64` & `aarch64`
- Linux `x86_64`

### Build from Source
```bash
cargo build --release
```
