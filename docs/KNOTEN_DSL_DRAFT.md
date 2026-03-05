# Knoten-DSL: The Evolution of Syntax (Draft)

*Status: Proposal / Sprint 57*

## Motivation
Currently, KnotenCore relies on handwritten JSON or transpiled output to represent its AST. While this is excellent for Agent-Native programming (AI generators), it is stressful and error-prone for human developers building or debugging complex UIs. 

We propose a human-readable, closure-based DSL (Domain-Specific Language) inspired by modern declarative UI frameworks (like SwiftUI and Jetpack Compose) but tailored to the KnotenCore architecture.

## 1. Core Principles
- **Curly-Brace Blocks**: `{}` naturally represent `Node::Block`.
- **Property Functions**: UI nodes take configuration arguments via standard function syntax `name(args)`.
- **Closures/Children**: UI nodes that can contain children take a trailing block `{ ... }`.
- **Event Callbacks**: Expressions responding to events use an arrow notation `-> { ... }`.

## 2. Syntax Examples

### 2.1 UI Styling & Grid Layout
```knoten
style(rounding: 20.0, spacing: 8.0, accent: [0, 0.5, 1, 1], fill: [0.1, 0.1, 0.1, 1]) {
    grid(cols: 4, id: "calc_grid") {
        button("7") -> { current = current * 10 + 7 }
        button("8") -> { current = current * 10 + 8 }
        button("9") -> { current = current * 10 + 9 }
        button("/") -> { op = "/"; last = current; current = 0 }
    }
}
```
*Ast Mapping:*
- `style(...) { ... }` maps directly to `Node::UISetStyle(rounding, spacing, accent, fill, None, None)` containing the inner block.
- `grid(...) { ... }` maps to `Node::UIGrid(4, "calc_grid", Block(...))`.
- `button("7") -> { ... }` maps to an `If(UIButton("7"), Block(...))`.

### 2.2 Scroll Areas & Fullscreen Panels
```knoten
fullscreen {
    scroll_area(id: "main_content") {
        label("Welcome to KnotenCore V2!")
        horizontal {
            button("Accept") -> { accept_tos = true }
            button("Decline") -> { exit() }
        }
    }
}
```
*Ast Mapping:*
- `fullscreen` maps to `Node::UIFullscreen`.
- `scroll_area` maps to `Node::UIScrollArea`.
- `horizontal` maps to `Node::UIHorizontal`.

## 3. Data & State Management
Variable assignment and mathematical operations remain C-like for familiarity. Arrays and Maps are initialized simply.

```knoten
let player_name = "Holger";
let scores = [100, 250, 42];

if (scores[0] > 50) {
    label("High Score!");
}
```

## 4. Parser Strategy (Next Steps)
To implement this DSL without breaking the existing Rust execution engine:
1. **Lexer/Parser:** Build a recursive descent parser (e.g., using `logos` and `chumsky` or via custom iteration) inside a new `src/parser.rs`.
2. **Intermediate AST:** The parser will output the exact same `ast::Node` enum currently strictly mapped to JSON.
3. **Execution:** The execution pipeline remains ignorant of whether the source was JSON or DSL:
   `DSL String -> Parser -> ast::Node -> Executor`.
