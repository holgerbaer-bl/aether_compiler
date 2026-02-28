# KnotenCore Rust-Ingestor Architecture (FFI Automation)

**Sprint 27: The Rust-Connector Automation** introduces autonomous expansion mechanics for KnotenCore. By deploying an ingestion framework over the engine, Aether logic can absorb natively compiled external programs.

## 1. The `rust_ingest` Pipeline

KnotenCore features a standalone AST extraction tool placed at `src/bin/rust_ingest.rs`. 
This pipeline reads arbitrary `.rs` language files and attempts to digest externally mapped functions, automating the creation of strictly-typed bridging headers (`ExternCall`).

### Execution
```bash
cargo run --bin rust_ingest src/test_lib.rs
```

### Process
1. **Targeting**: The ingestion pipeline queries the target Rust file for `pub fn <name>(<args>) -> <type>` declarations.
2. **Translation**: The arguments and structural layout are converted natively into Aether's Abstract Syntax Tree via the `Node::ExternCall` wrapper.
3. **Module Generation**: An `.nod` (Aether Executable Core) header JSON artifact is rendered to disk matching the target library's namespace.

## 2. The Native Execution Bridge (`bridge.rs`)

When an KnotenCore script calls `Import("test_lib.nod")`, it loads generated `ExternCall` nodes into its execution tree.

1. **Routing Strategy**: The interpreter triggers a context boundary swap during evaluate when hitting `ExternCall` matching structural traits.
2. **Registry Mapping**: A physical `BridgeModule` defined within the compiler source structure attempts to dynamically look up the ingested logic. 
3. **Rust Interception**: By utilizing the macro environment, the native memory closure executes seamlessly, yielding fully native computation performance dynamically over dynamically mapped interpreter pipelines.

## 3. Invocation Mechanics (Integration)
A script simply `Import`s the generated specification, then routes logic implicitly as if it were defined internally using local variable spaces.

```json
{
  "Block": [
    { "Import": "examples/core/test_lib.nod" },
    {
      "Assign": [
        "hash_value",
        {
          "Call": [
            "calculate_hash",
            [
              { "StringLiteral": "Hello World" }
            ]
          ]
        }
      ]
    }
  ]
}
```

The data flawlessly transfers from KnotenCore memory, triggers natively over the FFI border returning execution safely backward into the managed script container boundaries.

## 4. Complex Structs (Sprint 28)

The Ingestor now supports `pub struct` definitions. When a Rust file contains:

```rust
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn normalize_vector(v: Vector3) -> Vector3 { ... }
```

The ingestor generates:
1. **Constructor Function** — `Vector3(x, y, z)` returning a `Node::ObjectLiteral` with the struct's fields mapped as key-value pairs inside a `RelType::Object(HashMap)`.
2. **ExternCall Wrapper** — `normalize_vector(v)` remains an `ExternCall`, passing the Aether Object directly.

### Struct Marshalling (bridge.rs)

When the executor hits an `ExternCall` whose arguments include a `RelType::Object`, the bridge:
1. **Validates** all required fields exist and have the correct types. Missing fields trigger a clean `[FFI Error]` runtime fault.
2. **Unpacks** the HashMap into the native Rust struct (`Vector3 { x, y, z }`).
3. **Executes** the native function.
4. **Repacks** the returned struct back into a `RelType::Object(HashMap)`.

### Property Access

KnotenCore scripts can read individual fields from returned objects using `PropertyGet`:

```json
{ "PropertyGet": [ { "Identifier": "normalized" }, "x" ] }
```

### Example

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

Output: `0.6 (f64)` — confirming the struct was marshalled to Rust, normalized, and the result unpacked back into KnotenCore memory.
