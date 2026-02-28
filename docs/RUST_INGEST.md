# AetherCore Rust-Ingestor Architecture (FFI Automation)

**Sprint 27: The Rust-Connector Automation** introduces autonomous expansion mechanics for AetherCore. By deploying an ingestion framework over the engine, Aether logic can absorb natively compiled external programs.

## 1. The `rust_ingest` Pipeline

AetherCore features a standalone AST extraction tool placed at `src/bin/rust_ingest.rs`. 
This pipeline reads arbitrary `.rs` language files and attempts to digest externally mapped functions, automating the creation of strictly-typed bridging headers (`ExternCall`).

### Execution
```bash
cargo run --bin rust_ingest src/test_lib.rs
```

### Process
1. **Targeting**: The ingestion pipeline queries the target Rust file for `pub fn <name>(<args>) -> <type>` declarations.
2. **Translation**: The arguments and structural layout are converted natively into Aether's Abstract Syntax Tree via the `Node::ExternCall` wrapper.
3. **Module Generation**: An `.aec` (Aether Executable Core) header JSON artifact is rendered to disk matching the target library's namespace.

## 2. The Native Execution Bridge (`bridge.rs`)

When an AetherCore script calls `Import("test_lib.aec")`, it loads generated `ExternCall` nodes into its execution tree.

1. **Routing Strategy**: The interpreter triggers a context boundary swap during evaluate when hitting `ExternCall` matching structural traits.
2. **Registry Mapping**: A physical `BridgeModule` defined within the compiler source structure attempts to dynamically look up the ingested logic. 
3. **Rust Interception**: By utilizing the macro environment, the native memory closure executes seamlessly, yielding fully native computation performance dynamically over dynamically mapped interpreter pipelines.

## 3. Invocation Mechanics (Integration)
A script simply `Import`s the generated specification, then routes logic implicitly as if it were defined internally using local variable spaces.

```json
{
  "Block": [
    { "Import": "examples/core/test_lib.aec" },
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

The data flawlessly transfers from AetherCore memory, triggers natively over the FFI border returning execution safely backward into the managed script container boundaries.
