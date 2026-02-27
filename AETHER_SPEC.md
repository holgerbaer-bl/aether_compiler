# AetherCore Binary AST Specification (v1.0)

AetherCore is a native abstract syntax tree (AST) programming language for AI systems. It bypasses text-parsing and instead consumes highly-efficient serialized binary AST structures.

This specification documents the exact binary structural layout of the valid AetherCore abstract syntax tree. The execution environment deserializes this layout directly into executable logic.

## 1. Serialization Format
AetherCore AST schemas are serialized using **Bincode** (Little-Endian, fixed integer sizes). All Aether programs are distributed as `.aec` files (AetherCore Executable).

## 2. Core Execution Model
AetherCore executes structurally. Programs are represented by the `Node` enum. Each compilation unit or script starts with an implicit root block `Node::Block(Vec<Node>)` or any single `Node`.

## 3. Data Types
AetherCore defines the following base types for AST values, managed as dynamically typed registers inside the runtime memory state:
- **Int**: 64-bit signed integer (`i64`)
- **Float**: 64-bit floating point number (`f64`)
- **Bool**: 8-bit boolean value (`true = 1`, `false = 0`)
- **String**: UTF-8 string, prefixed with a 64-bit length identifier

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
*   **`Eq(Box<Node>, Box<Node>)`**: Logical equality comparison.
*   **`Lt(Box<Node>, Box<Node>)`**: Less than comparison.

### 4.4. Control Flow
*   **`If(Box<Node>, Box<Node>, Option<Box<Node>>)`**: Evaluates the first `Node` (Condition). If true, executes the second `Node` (Then Branch). Otherwise executes the third optional `Node` (Else Branch).
*   **`While(Box<Node>, Box<Node>)`**: Evaluates the first `Node`. While true, repeatedly executes the second `Node` (Body block).
*   **`Block(Vec<Node>)`**: Unconditionally executes a sequence of nodes in order. The block returns the value of its last node, or implicit void if empty.
*   **`Return(Box<Node>)`**: Exits the current execution context (or program) returning the evaluated Node's result.

## 5. Execution State & Return Value
Upon execution of a `.aec` structure, the engine evaluates nodes from root to leaf. 
The program's outcome is the value of the explicit root `Return` node, or the value of the last node in the top-level block.

## 6. Binary Footprint (Bincode Enum tags)
The `bincode` deserializer maps Rust Enums linearly starting from index `0u32`. Thus, the binary header for a `Node` identifies its enum variant as a 32-bit integer, followed by the flattened representation of its inner fields.
