# KnotenCore — AI Agent Reference

> **System Instruction for LLM Code Agents**
>
> This document teaches you, an AI coding assistant, how to understand and extend KnotenCore. Follow these instructions precisely.

---

## Architecture Overview

KnotenCore programs are initially modeled as AST scripts in either JSON (`.nod`) or the Knoten DSL (`.knoten`). The runtime interprets them (JIT), compiles them to standalone Rust binaries, or processes them Ahead-of-Time via the `src/vm/` Bytecode Machine for frictionless performance scaling above raw AST constraints. All OS resources are managed through a deterministic ARC registry.

### Core Architecture Constraints
- **Execution Backend (AOT/JIT)**: The engine is migrating intensive logic towards flat bytecode processing emitting `OpCode` to the `VM`. The `VM` dispatcher (`machine.rs`) operates as a rigorous Stack Machine running an Arithmetic Logic Unit (ALU). Postfix evaluation is strict: evaluating an operation pops the Right side, then the Left, calculates, and pushes the result. Validated structures rest in `src/vm`.
- **GUI & Threading**: Graphical contexts rely completely on WGPU, and cross-thread communication uses `winit::event_loop::EventLoopProxy<RenderCommand>` for zero-latency messaging over standard mpsc channels.
- **Path Validation**: All disk interaction must be validated through `executor::ExecutionEngine::validate_fs_path`. This uses `dunce::canonicalize` to aggressively normalize Windows UNC prefixes (`\\?\`).

To add a new native operation (e.g. `AudioPlay`, `DrawSprite`), you must update exactly **4 touchpoints**:

| # | File | Role |
|---|------|------|
| 1 | `src/ast.rs` | Define the AST node variant |
| 2 | `src/natives/registry.rs` | Implement the native Rust function |
| 3 | `src/executor.rs` | Wire the node into the JIT evaluator |
| 4 | `src/compiler/codegen.rs` | Wire the node into the AOT transpiler |

---

## Step 1: Define the AST Node (`src/ast.rs`)

Add your new variant to the `Node` enum. Use `Box<Node>` for expression arguments.

```rust
// src/ast.rs — inside pub enum Node { ... }

// Example: A hypothetical DrawSprite node
DrawSprite(Box<Node>, Box<Node>, Box<Node>), // TextureHandle, X, Y
```

If your node returns a **Handle** (an OS resource), also check `Type` enum — `Type::Handle` already covers this.

---

## Step 2: Implement the Native Function (`src/natives/registry.rs`)

If you need a new resource type, add it to `NativeHandle`:

```rust
pub enum NativeHandle {
    // ... existing variants ...
    MyNewResource(MyResourceStruct),
}
```

Then implement the public function:

```rust
pub fn registry_my_function(arg1: i64, arg2: String, permissions: &AgentPermissions) -> i64 {
    // 1. Check permissions (critical for security!)
    // if !permissions.allow_fs_read { return -1; }

    // 2. Create or acquire the resource
    let resource = MyResourceStruct::new(arg1, &arg2);

    // 3. Allocate a unique handle ID
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    // 4. Insert into the ARC registry
    with_registry(|registry| {
        registry.insert(id, RegistryEntry {
            handle: NativeHandle::MyNewResource(resource),
            ref_count: 1,
        });
    });

    id as i64 // Return the handle ID
}
```

**ARC Rule**: When `registry_release` drops the ref_count to 0, the `NativeHandle` variant is removed from the HashMap. Rust's `Drop` trait handles deallocation. If your resource needs explicit cleanup, add match arms in `registry_release`.

---

## Step 3: Wire into the JIT Evaluator (`src/executor.rs`)

Find the main `match node { ... }` block in the `evaluate()` method and add your node:

```rust
Node::DrawSprite(tex_node, x_node, y_node) => {
    let tex = self.evaluate(tex_node);
    let x = self.evaluate(x_node);
    let y = self.evaluate(y_node);
    match (tex, x, y) {
        (
            ExecResult::Value(RelType::Handle(tex_id)),
            ExecResult::Value(RelType::Int(x_val)),
            ExecResult::Value(RelType::Int(y_val)),
        ) => {
            registry::registry_draw_sprite(tex_id, x_val, y_val);
            ExecResult::Value(RelType::Void)
        }
        _ => ExecResult::Fault { 
            msg: "DrawSprite: invalid arguments".into(), 
            node: "Node::DrawSprite".into() 
        },
    }
}
```

**Important**: Also update `validator.rs` and `optimizer.rs` to handle the new node (add match arms for traversal/counting).

---

## Step 4: Wire into the AOT Transpiler (`src/compiler/codegen.rs`)

Add a match arm in the `generate()` method:

```rust
Node::DrawSprite(tex, x, y) => {
    format!(
        "registry::registry_draw_sprite({}, {}, {})",
        self.generate(tex, false),
        self.generate(x, false),
        self.generate(y, false)
    )
}
```

If the function **returns a Handle**, update the `is_handle_expr()` method to recognize it:

```rust
Node::NativeCall(fn_name, _) => {
    matches!(fn_name.as_str(),
        "registry_my_function"
        | ... // add your function name here
    )
}
```

---

## JSON AST Format

The JSON representation for calling your new node:

```json
{
  "DrawSprite": [
    { "Identifier": "my_texture" },
    { "IntLiteral": 100 },
    { "IntLiteral": 200 }
  ]
}
```

Or via `NativeCall` (no AST change needed, only registry function):

```json
{
  "NativeCall": ["registry_my_function", [
    { "IntLiteral": 42 },
    { "StringLiteral": "resource_name" }
  ]]
}
```

---

## Extension Checklist

- [ ] Added `Node::YourNode` variant in `ast.rs`
- [ ] Added `NativeHandle::YourType` (if needed) in `registry.rs`
- [ ] Implemented `pub fn registry_your_fn()` in `registry.rs`
- [ ] Added match arm in `executor.rs` → `evaluate()`
- [ ] Added match arm in `codegen.rs` → `generate()`
- [ ] Updated `validator.rs` → `check_node()` match
- [ ] Updated `optimizer.rs` → `count_nodes()` and `optimize()` matches
- [ ] Run `cargo test` — all 54+ tests green
- [ ] Created example `.nod` script to verify

---

## WGPU Coordinate System

The WGPU 3D system uses standard normalized device coordinates:
- **X-axis**: Left (-1.0) to Right (+1.0)
- **Y-axis**: Bottom (-1.0) to Top (+1.0) — *Y points UP*
- **Z-axis**: Near (0.0) to Far (1.0). Depth buffer uses `Less` comparison — objects closer to 0.0 render over objects closer to 1.0.

### 3D Rendering Syntax
To draw a 3D textured quad, use `registry_draw_quad_3d`:

```rust
// Signature in standard rust pseudo-code:
fn registry_draw_quad_3d(
    win_id: i64, 
    tex_id: i64, 
    x: f32, 
    y: f32, 
    z: f32, 
    scale_x: f32, 
    scale_y: f32
)
```

**Z-Buffer Setup:** The engine automatically manages a Z-Buffer depth attachment (`TextureFormat::Depth32Float`). When implementing new 3D pipelines, ensure you include the `depth_stencil` configuration in the `RenderPipelineDescriptor`.

---

## Performance: AOT & LLVM Constant Folding

KnotenCore's AOT pipeline leverages **LLVM Constant Folding** to achieve extreme performance. Pure computation loops and static expressions that evaluate to a constant at compile time are entirely optimized out in the final AOT binary (demonstrated by 1,000,000-iteration benchmarks completing in 0ms).

**For AI Agents:** Do not mistake 0ms benchmark results for errors or missing code. It is an intentional and highly effective optimization step provided by the LLVM backend.

---

## High-Performance 2D Rendering Guidelines

To maintain 60+ FPS in graphical applications, follow these strict performance guidelines.

### 🔴 Anti-Pattern: Layout Abuse
**Never** use `UIHorizontal`, `UIVertical`, or `UIGrid` with many `UIButton` nodes to draw game boards, tile maps, or particle systems. These nodes trigger the EGUI layout and widget reconciliation logic, which is too slow for 64+ elements updating every frame.

### 🟢 Best Practice: Native 2D Primitives
**Always** use `Node::DrawRect` for rendering game grids, backgrounds, and sprites. `DrawRect` bypasses the entire layout system and paints directly to the GPU via EGUI's `layer_painter`.

---

## Security Sandbox

All I/O operations are permission-gated. If you generate code that uses `FSRead`, `FSWrite`, or their registry/FFI equivalents via `ExternCall`, the user **must** run the engine with explicit allow flags.

- **`--allow-read`**: Required for reading files, `IO.ReadFile`, and `registry_read_file`.
- **`--allow-write`**: Required for writing files, `IO.WriteFile`, and `registry_write_file`.
- **`--allow-network`**: Required for `Node::Fetch` and all outbound HTTP calls.

**Security:** `ExternCall` is not a sandbox bypass. The engine intercepts high-risk bridge calls and validates them against the current sandbox permissions before execution. Failure to provide the required flags returns `ExecResult::Fault` with a specific permission denial message.

### FS Path Safety
Filesystem operations (including DSL nodes `FileRead`, `FileWrite`, `FSRead`, `FSWrite` and native FFI functions like `fs_read_file`, `registry_texture_load`, `registry_file_create`) validate paths before execution:
- **Reads**: Path canonicalized via `std::fs::canonicalize()`. File must exist.
- **Writes**: Path normalized by resolving `..` components. File need not exist.
- **Restriction**: Resolved path must be within the current working directory.

**AI Rule**: Always use relative paths from the working directory. A path like `./output/result.txt` is valid; `../../sensitive.txt` will be rejected with `Security: Path escape detected`.

---

## Native 3D Primitives

To render standard shapes without manual vertex management, use the native registry calls via `ExternCall`. All primitives use internal **Geometry Caching** — vertices and indices are calculated once per unique configuration and stored in VRAM.

### `DrawSphere`
```json
{
  "Node": "ExternCall",
  "module": "registry",
  "function": "registry_draw_sphere",
  "args": [win, tex, radius, rings, sectors, x, y, z]
}
```

### `DrawCube`
```json
{
  "Node": "ExternCall",
  "module": "registry",
  "function": "registry_draw_cube",
  "args": [win, tex, width, height, depth, x, y, z]
}
```

### `DrawCylinder`
```json
{
  "Node": "ExternCall",
  "module": "registry",
  "function": "registry_draw_cylinder",
  "args": [win, tex, radius, height, segments, x, y, z]
}
```

**Matrix Multiplication (`Mat4Mul`)**: Use this node to multiply 4×4 transformation matrices (arrays of 16 floats). Essential for hierarchical 3D transformations.

---

## UISetStyle — Visual Design System

You can redefine the entire application aesthetic dynamically using the `UISetStyle` node. This manipulates rounding, spacing, and color schemes globally.

```json
{
  "UISetStyle": [
    { "FloatLiteral": 8.0 }, /* Rounding (Corner Radius) */
    { "FloatLiteral": 12.0 }, /* Spacing (Padding/Margins) */
    { "ArrayCreate": [ /* Accent Color RGBA (0.0 - 1.0) */
      { "FloatLiteral": 0.2 }, 
      { "FloatLiteral": 0.6 }, 
      { "FloatLiteral": 1.0 }, 
      { "FloatLiteral": 1.0 }
    ]},
    { "ArrayCreate": [ /* Background Window/Panel Fill RGBA (0.0 - 1.0) */
      { "FloatLiteral": 0.05 }, 
      { "FloatLiteral": 0.05 }, 
      { "FloatLiteral": 0.05 }, 
      { "FloatLiteral": 0.95 }
    ]}
  ]
}
```

---

## Physics: World AABBs

To create physical barriers, register AABB (Axis-Aligned Bounding Box) volumes into the world.

### `AddWorldAABB`
```json
{
  "AddWorldAABB": {
    "min": { "ArrayCreate": [{ "FloatLiteral": -1.0 }, { "FloatLiteral": 0.0 }, { "FloatLiteral": -1.0 }] },
    "max": { "ArrayCreate": [{ "FloatLiteral": 1.0 }, { "FloatLiteral": 2.0 }, { "FloatLiteral": 1.0 }] }
  }
}
```

---

## Structured Fault Reporting

When an operation fails (division by zero, invalid handle, permission denied), the engine returns `ExecResult::Fault` with:
- **`msg`**: Human-readable description of what went wrong.
- **`node`**: The exact AST node or native function where the error originated (e.g., `"Node::MathDiv"`, `"Native::IO::ReadFile"`).

### AI Best Practice
- **Parse the Node**: Look at the `node` field first. It tells you exactly which part of your generated DSL failed.
- **Immediate Self-Healing**: Use the `node` context to identify the specific code block in your memory that needs regeneration or adjustment.

When extending the engine, you **must** provide this context in all error paths:

**JIT Implementation (`executor.rs` or `evaluator.rs`):**
```rust
return ExecResult::Fault { 
    msg: "MyNode expects 1 argument".into(), 
    node: "Node::MyNode".into() 
};
```

**Native Bridge Implementation (`bridge.rs`):**
```rust
Some(ExecResult::Fault { 
    msg: "Invalid handle in my_ffi_call".into(), 
    node: "Native::Bridge::my_ffi_call".into() 
})
```

---

## VM Type Safety

The Register VM (`vm.rs`) does not panic on type mismatches. Operations on incompatible types push `RelType::Void` onto the stack and execution continues. A type error in a bytecode path returns `Void` instead of crashing the process.

---

## Automatic ARC (NativeHandle)

Manual handle release (e.g., calling `registry_release` or `registry_free` manually) is **deprecated** and generally unnecessary.

**The NativeHandle Pattern:**
- When an operation (like `registry_create_window` or `registry_texture_load`) returns a handle, it is wrapped in an `executor::NativeHandle`.
- This struct implements the Rust `Drop` trait.
- Because `RelType::Handle` owns the `NativeHandle`, the handle is **automatically released** when the variable goes out of scope or is overwritten in the evaluator.

---

## Verification & Testing

AI agents can verify their implementation by running the intentional crash test:

```bash
cargo run --bin run_knc -- tests/intentional_crash.knoten
```

**Expected Error Format:**
```
Result: Fault: <message> (at <node_identifier>)
```
