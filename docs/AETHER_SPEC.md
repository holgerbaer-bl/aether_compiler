# AetherCore Binary AST Specification (v1.0)

AetherCore is a native abstract syntax tree (AST) programming language for AI systems. It bypasses text-parsing and instead consumes highly-efficient serialized binary AST structures.

This specification documents the exact binary structural layout of the valid AetherCore abstract syntax tree. The execution environment deserializes this layout directly into executable logic.

## 1. Serialization Format
AetherCore AST schemas are serialized using **Bincode** (Little-Endian, fixed integer sizes). All Aether programs are distributed as `.aec` files (AetherCore Executable).

## 2. Core Execution Model
AetherCore executes structurally. Programs are represented by the `Node` enum. Each compilation unit or script starts with an implicit root block `Node::Block(Vec<Node>)` or any single `Node`. 

The runtime maintains a **Call Stack** of **Stack Frames**. Variable resolution always prioritizes the local `StackFrame` before falling back to the global state.

## 3. Data Types
AetherCore defines the following base types for AST values, managed as dynamically typed registers inside the runtime memory state, but statically locked during compilation by the internal `TypeChecker`:
- **Int**: 64-bit signed integer (`i64`)
- **Float**: 64-bit floating point number (`f64`)
- **Bool**: 8-bit boolean value (`true = 1`, `false = 0`)
- **String**: UTF-8 string, prefixed with a 64-bit length identifier
- **Array**: A dynamically sized list of values
- **Object**: Dictionary key mapping formats.
- **Void**: Null expression boundaries.
- **Any**: Unresolved variable signatures gracefully degrading type inferences.

## 4. AST Node Structures
The AST consists of a single sum type `Node`, defined mathematically as follows:

### 4.1. Literals (Values)
*   **`IntLiteral(i64)`**: A constant 64-bit integer.
*   **`FloatLiteral(f64)`**: A constant 64-bit float.
*   **`BoolLiteral(bool)`**: A constant boolean.

### 4.2. Memory Operations
*   **`Assign(Identifier, Box<Node>)`**: Evaluates the right-hand `Node` and assigns the result to the variable `Identifier` (a String) in the current scope.
*   **`Identifier(String)`**: Dereferences a variable by name. Returns a runtime fault if undefined.

### 4.3. Mathematical & Logical Operations
Operations take a left-hand side (`lhs`) and right-hand side (`rhs`).
*   **`Add(Box<Node>, Box<Node>)`**: Mathematical addition (int + int, float + float, etc.).
*   **`Sub(Box<Node>, Box<Node>)`**: Mathematical subtraction.
*   **`Mul(Box<Node>, Box<Node>)`**: Mathematical multiplication.
*   **`Div(Box<Node>, Box<Node>)`**: Mathematical division.
*   **`Sin(Box<Node>)`**: Returns the Sine of a `Float`.
*   **`Cos(Box<Node>)`**: Returns the Cosine of a `Float`.
*   **`Mat4Mul(Box<Node>, Box<Node>)`**: Multiplies two 16-element Float Arrays (Column-Major 4x4 Matrices) and returns the resulting 16-element Float Array.
*   **`Time()`**: Returns the monotonic application runtime in seconds as a `Float`.
*   **`Eq(Box<Node>, Box<Node>)`**: Logical equality comparison.
*   **`Lt(Box<Node>, Box<Node>)`**: Less than comparison.

### 4.4. Functions and Scoping
*   **`FnDef(String, Vec<String>, Box<Node>)`**: Defines a function. Identifier, parameter names, and body Block.
*   **`Call(String, Vec<Node>)`**: Calls a function by identifier with arguments.

*   **`ArrayLiteral(Vec<Node>)`**: Instantiates a new array.
*   **`ArrayGet(String, Box<Node>)`**: Retrieves an element from a variable at the given index.
*   **`ArraySet(String, Box<Node>, Box<Node>)`**: Sets an element in a variable at the given index.
*   **`ArrayPush(String, Box<Node>)`**: Appends an evaluated value to the end of the specified array.
*   **`ArrayLen(String)`**: Returns the length of an array or string as an `Int`.
*   **`Index(Box<Node>, Box<Node>)`**: Accesses an element in an array or string at a given index.
*   **`Concat(Box<Node>, Box<Node>)`**: Concatenates two strings or two arrays.

### 4.6. Bitwise Operations (for Serialization)
*   **`BitAnd(Box<Node>, Box<Node>)`**: Bitwise AND on Ints.
*   **`BitShiftLeft(Box<Node>, Box<Node>)`**: Left shift `<<` on Ints.
*   **`BitShiftRight(Box<Node>, Box<Node>)`**: Right shift `>>` on Ints.

### 4.7. File I/O
*   **`FileRead(Box<Node>)`**: Reads a file by path. Returns file contents as an Array of Int bytes.
*   **`FileWrite(Box<Node>, Box<Node>)`**: Writes an Array of Int bytes (arg 2) to a file path (arg 1).
*   **`Print(Box<Node>)`**: Evaluates the node and prints the resulting value to the system terminal (stdout).
*   **`NativeCall(String, Vec<Node>)`**: Invokes a built-in "Native" function.
    - `Math.Random`: Returns a float between 0.0 and 1.0.
    - `Math.Sin`, `Math.Cos`: Standard trigonometric functions.
    - `Math.Floor`, `Math.Ceil`: Standard rounding functions.
    - `Math.Perlin2D`: Returns a Perlin noise float based on `(x, y)` coordinates.
    - `IO.WriteFile(path, content)`: Writes `content` (String) to `path` (String). Returns a Boolean.
    - `IO.ReadFile(path)`: Reads the file at `path` (String) and returns its contents as a String.
    - `IO.AppendFile(path, content)`: Appends `content` (String) to the file at `path` (String). Returns a Boolean.
    - `IO.FileExists(path)`: Returns `true` if the file at `path` (String) exists on disk, `false` otherwise.
*   **`ExternCall { module: String, function: String, args: Vec<Node> }`**: Bridging structure for explicitly typed Foreign Function Interfaces out to native C/Rust libraries. Arguments mapped via strictly enforced static type assignments.

### 4.8. 3D Graphics (Vulkan/Metal/DX12 via WGPU)
*   **`InitWindow(Box<Node>, Box<Node>, Box<Node>)`**: Initializes an OS Window (Width, Height, Title). Opens the window on the system.
*   **`InitGraphics()`**: Initializes the GPU Adapter and Device.
*   **`LoadShader(Box<Node>)`**: Compiles a WGSL Shader from a String. Returns a Shader Identifier.
*   **`RenderMesh(Box<Node>, Box<Node>, Box<Node>)`**: Executes a RenderPass draw call to the screen (Shader Identifier, Vertex Buffer Array, MVP Matrix Uniform Array).
*   **`PollEvents(Box<Node>)`**: Submits a block of nodes to run inside the Window Event Loop, intercepting close requests.

#### `InitCamera`
Initializes a First-Person 3D Camera mapping keyboard `WASD` inputs to matrix projections automatically.
*   **Structure:** `{"InitCamera": [{"FloatLiteral": <FOV>}]}`

#### `DrawVoxelGrid`
Receives an array of voxel positions defining chunks of blocks in 3-dimensional space (instanced geometry).
*   **Structure:** `{"DrawVoxelGrid": [{"ArrayLiteral": [X, Y, Z, BlockID, X, Y, Z, BlockID, ...]}]}`

*   **Structure:** `{"LoadTextureAtlas": [{"StringLiteral": "<Filepath>"}, {"FloatLiteral": <TileSize (e.g. 16.0)>}]}`

#### `InitVoxelMap`
Switches the voxel renderer from static instance arrays to a persistent, mutable internal `HashMap` state. This is required for real-time mining and building.
*   **Structure:** `"InitVoxelMap"`

#### `EnableInteraction`
Activates 3D raycasting (DDA algorithm) and mouse input listeners. Left-click breaks blocks (Mining), Right-click places blocks (Building).
*   **Structure:** `{"EnableInteraction": [{"BoolLiteral": <true/false>}]}`

#### `EnablePhysics`
Activates real-time player physics, including gravity, AABB collision against the Voxel Map, and jumping (Spacebar).
*   **Structure:** `{"EnablePhysics": [{"BoolLiteral": <true/false>}]}`

#### `SetVoxel`
Directly modifies the persistent Voxel Map at the specified coordinates.
*   **Standard Block IDs & Atlas Layout (4x4 tiles):**
    - 1: **Grass** (Top: Tile 0, Side: Tile 1, Bottom: Tile 2/Dirt)
    - 2: **Stone** (All: Tile 3)
    - 3: **Sand** (All: Tile 4)
    - 4: **Water** (All: Tile 5)
    - 5: **Wood** (Top/Bottom: Tile 7, Side: Tile 6)
    - 6: **Leaves** (All: Tile 8)
*   **Structure:** `{"SetVoxel": [<X>, <Y>, <Z>, <BlockID>]}`

#### `LoadSample`
Reads an audio file into system RAM buffering bytes natively using the Rodio CPAL interface (Amiga Paula mapping).
*   **Structure:** `{"LoadSample": [{"IntLiteral": <SampleID>}, {"StringLiteral": "<Filepath>"}]}`

#### `PlaySample`
Triggers an asynchronous, polyphonic audio stream natively out of the thread loop. Pitch and Volume map natively.
*   **Structure:** `{"PlaySample": [{"IntLiteral": <SampleID>}, {"FloatLiteral": <Volume>}, {"FloatLiteral": <Pitch>}]}`

### 4.9. 8-Bit Audio Engine (CPAL FFI)
*   **`InitAudio()`**: Bootstraps the `cpal` low-latency audio stream and software synthesizer.
*   **`PlayNote(Box<Node>, Box<Node>, Box<Node>)`**: Starts synthesizing a tone. (Channel: Integer 0-3, Frequency: Float Hz, Waveform: Integer 0-4) (0: Sine, 1: Square, 2: Sawtooth, 3: Triangle, 4: Noise).
*   **`StopNote(Box<Node>)`**: Mutes the specified Channel.

### 4.10. The Asset Pipeline (PS2-Era Data)
*   **`LoadMesh(Box<Node>)`**: Parses an `.obj` file from the given Path. Returns a `MeshID`.
*   **`LoadTexture(Box<Node>)`**: Parses a `.png` file from the given Path. Returns a `TextureID`.
*   **`PlayAudioFile(Box<Node>)`**: Loads a `.wav` file and streams it dynamically into the active CPAL audio buffer.
*   **`RenderAsset(Box<Node>, Box<Node>, Box<Node>, Box<Node>)`**: Takes `ShaderID`, `MeshID`, `TextureID`, and `UniformArray`. Renders textured models over WGPU.

### 4.11. UI & Text Engine (Office/2D Canvas)
*   **`LoadFont(Box<Node>)`**: Takes a Path String to a `.ttf` file. Inflates it into the WGPU Glyph structure.
*   **`DrawText(Box<Node>, Box<Node>, Box<Node>, Box<Node>, Box<Node>)`**: Takes `Text` (String), `X` (Float), `Y` (Float), `Size` (Float), and `Color` (Array of 4 Floats: R,G,B,A). Queues 2D text onto the screen for the current Frame.
*   **`GetLastKeypress()`**: Retrieves and clears the engine's internal keyboard buffer, returning a `String` containing the characters typed since the last check.

### 4.12. Desktop GUIs (Egui Immediate Mode)
*   **`UIWindow(Box<Node>, Box<Node>)`**: Takes a Title (String) and a Body (Block Node). Spawns an interactive, draggable OS-styled window within the WGPU canvas and evaluates the inner Body block to populate its contents.
*   **`UILabel(Box<Node>)`**: Takes Text (String) and renders a formatted text label within the current UI context.
*   **`UIButton(Box<Node>)`**: Takes Text (String). Renders a clickable button in the UI context. Returns `Int` 1 if clicked this frame, 0 otherwise.
*   **`UITextInput(Box<Node>)`**: Takes a Variable Name (String). Instantiates a single-line text input field inextricably linked to that variable in the ambient memory store, reacting to keyboard polling and cursor selection automatically.

### 4.13. Control Flow
*   **`Import(String)`**: Imports another AetherCore executable JSON file by path, making its top-level definitions and variable assignments available in the current global scope.
*   **`If(Box<Node>, Box<Node>, Option<Box<Node>>)`**: Evaluates the first `Node` (Condition). If true, executes the second `Node` (Then Branch). Otherwise executes the third optional `Node` (Else Branch).
*   **`While(Box<Node>, Box<Node>)`**: Evaluates the first `Node`. While true, repeatedly executes the second `Node` (Body block).
*   **`Block(Vec<Node>)`**: Unconditionally executes a sequence of nodes in order. The block returns the value of its last node, or implicit void if empty.
*   **`Return(Box<Node>)`**: Exits the current execution context (or program) returning the evaluated Node's result.

## 5. Execution State & Return Value
Upon execution of a `.aec` structure, the engine evaluates nodes from root to leaf. 
The program's outcome is the value of the explicit root `Return` node, or the value of the last node in the top-level block.

## 6. Binary Footprint & Bundling
The `run_aec` executor actively checks for the `AETHER_BUNDLE` environment flag during execution routines. The local toolchain exposes the `aether_build <file.json>` build command which evaluates custom memory directives hooking the AST natively within machine code. This outputs standalone `.exe` packages for zero-dependency execution.
## 5. AI Safety & Validation

AetherCore implements a multi-layer validation strategy to ensure that AI-generated scripts are syntactically and logically sound before execution.

### 5.1. JSON Schema (`aether_schema.json`)
A formal JSON Schema is provided in the repository root. This schema defines the structural requirements for every `Node` variant. Developers and AI agents should use this schema for real-time validation during the code synthesis phase.

### 5.2. Pre-Flight Validator
The `run_aec` tool includes a built-in static analyzer that performs deep AST inspection. It detects:
- **Empty Identifiers**: Ensures function and variable names are populated.
- **File Integrity**: Verifies that all `Import` paths resolve to existing files.
- **Circular Imports**: Detects and prevents infinite recursion in modular codebases.
- **Structural Integrity**: Ensures that complex nodes (like `FnDef` or `Call`) have the required sub-nodes.

### 5.3. CLI Validation Mode
Users can validate any script without executing it by using the `--check` flag:
```bash
cargo run --bin run_aec -- --check examples/voxel/showcase_world.json
```
If the script passes all checks, the tool outputs `Syntax OK`. Otherwise, it provides a detailed list of logical and structural errors.
