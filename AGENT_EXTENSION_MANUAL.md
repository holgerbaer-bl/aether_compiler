# KnotenCore — Agent Extension Manual

> **System Instruction for LLM Code Agents**
>
> This document teaches you, an AI coding assistant, how to extend KnotenCore with new native operations. Follow these steps precisely.

---

## Architecture Overview

KnotenCore programs are JSON-encoded Abstract Syntax Trees (AST). The runtime interprets them (JIT) or compiles them to standalone Rust binaries (AOT). All OS resources are managed through a deterministic ARC registry.

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
pub fn registry_my_function(arg1: i64, arg2: String) -> i64 {
    // 1. Create or acquire the resource
    let resource = MyResourceStruct::new(arg1, &arg2);

    // 2. Allocate a unique handle ID
    let mut id_guard = COUNTER_NEXT_ID.lock().unwrap();
    let id = *id_guard;
    *id_guard += 1;

    // 3. Insert into the ARC registry
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

## Checklist

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

## Sprint 67: High-Performance 2D & Game Engine Evolution

KnotenCore has evolved from a pure UI execution engine into a **hybrid Game Engine**. To maintain 60+ FPS in graphical applications, follow these strict performance guidelines.

### 🔴 Anti-Pattern: Layout Abuse
**Never** use `UIHorizontal`, `UIVertical`, or `UIGrid` with many `UIButton` nodes to draw game boards, tile maps, or particle systems. 
- **Reason**: These nodes trigger the EGUI layout and widget reconciliation logic, which is too slow for 64+ elements updating every frame.

### 🟢 Best Practice: Native 2D Primitives
**Always** use `Node::DrawRect` for rendering game grids, backgrounds, and sprites.
- **Reason**: `DrawRect` bypasses the entire layout system and paints directly to the GPU via EGUI's `layer_painter`. It is designed for "drawing," not "layout."

**Example: Efficient 8x8 Grid**
```json
{
  "While": [ ... loop 64 times ...
    {
      "DrawRect": {
        "x": { "Identifier": "px" },
        "y": { "Identifier": "py" },
        "width": { "IntLiteral": 32 },
        "height": { "IntLiteral": 32 },
        "color": [0.1, 0.8, 0.2, 1.0]
      }
    }
  ]
}
```

### Layout Hybridization
Use `UIFillParent` inside a `UIFullscreen` or `UIWindow` to claim the entire rendering area before starting your `While` loops for `DrawRect`. Use `UIFixed` if you need to reserve a specific pixel-stable area within a responsive UI.

---

## Sprint 78: Error Tracing Foundation (Diagnostic Context)

To enable robust **Self-Healing**, the engine now provides structured error reports.

### 1. The Structured `Fault`
When an operation fails (e.g., division by zero, invalid handle, permission denied), the engine returns an `ExecResult::Fault` with two fields:
- **`msg`**: A human-readable description of what went wrong.
- **`node`**: The exact AST node or native function where the error originated (e.g., `"Node::MathDiv"`, `"Native::IO::ReadFile"`).

### 2. Implementation for AI Agents
When extending the engine, you **must** provide this context. Avoid generic error strings.

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

### AI Best Practice:
- **Parse the Node**: When you receive an error, look at the `node` field first. It tells you exactly which part of your generated DSL failed, bypassing the need to "guess" based on the error message alone.
- **Immediate Self-Healing**: Use the `node` context to identify the specific code block in your memory that needs regeneration or adjustment.
