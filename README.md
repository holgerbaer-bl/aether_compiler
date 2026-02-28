# AetherCore

AetherCore is a high-performance, JIT-compiled runtime environment designed for AI-native execution. It processes a turing-complete JSON Abstract Syntax Tree (AST) directly, eliminating the overhead of text-based parsing for large language models and autonomous agents.

## Core Philosophy

AetherCore is built on the principle that AI agents should not be constrained by human-readable syntax. By utilizing a structured JSON AST as its primary language, AetherCore provides a direct interface for agents to generate, optimize, and execute complex logic with maximum precision and zero parsing ambiguity.

## Technical Specifications

- **AI-Native Language Interface**: Pure JSON-based AST for direct model-to-code synthesis.
- **AI Safety & Validation**: Built-in AST validation and a formal JSON Schema to ensure code integrity before execution.
- **WGPU-Accelerated Rendering**: A high-performance graphics backend for compute-heavy and visual applications.
- **Modular Architecture**: Comprehensive support for modular codebases via a native import system.
- **Turing-Complete Control Flow**: Integrated support for recursion, loops (`While`), and conditional branching (`If`).
- **Standard Library (StdLib)**: Optimized Rust-native implementations for mathematics, bitwise operations, and memory management.

## Technical Showcase: Voxel Engine POC

The repository includes a comprehensive Voxel rendering engine as a proof-of-concept for AetherCore's capabilities. This showcasing world demonstrates:
- Persistent state management across large-scale voxel maps.
- Real-time player physics and AABB collision detection.
- Advanced WGSL shader integration with directional lighting and distance-based fog.
- Procedural terrain generation utilizing native Perlin noise.

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/) (Latest Stable)
- A GPU compatible with Vulkan, Metal, or DX12.

### Execution
To execute an AetherCore script:
```bash
cargo run --bin run_aec <path_to_script.json>
```

---

# AetherCore (Deutsch)

AetherCore ist eine leistungsoptimierte JIT-Laufzeitumgebung, die speziell für die KI-native Ausführung entwickelt wurde. Sie verarbeitet einen Turing-vollständigen JSON Abstract Syntax Tree (AST) direkt, wodurch der Overhead durch Text-Parsing für LLMs und autonome Agenten entfällt.

## Kernphilosophie

AetherCore basiert auf dem Prinzip, dass KI-Agenten nicht durch menschenlesbare Syntax eingeschränkt werden sollten. Durch die Nutzung eines strukturierten JSON-AST als Primärsprache bietet AetherCore eine direkte Schnittstelle für Agenten, um komplexe Logik mit maximaler Präzision und ohne Parsing-Ambiguität zu generieren und auszuführen.

## Technische Highlights

- **KI-Native Schnittstelle**: Reine JSON-basierte AST-Struktur für die direkte Code-Synthese durch Modelle.
- **KI-Sicherheit & Validierung**: Integrierte AST-Validierung und ein formales JSON-Schema zur Sicherstellung der Code-Integrität vor der Ausführung.
- **WGPU-Grafikbeschleunigung**: Hochleistungs-Backend für rechenintensive und visuelle Anwendungen.
- **Modulare Architektur**: Unterstützung für skalierbare Codebasen durch ein natives Import-System.
- **Turing-Vollständigkeit**: Native Unterstützung für Rekursion, Schleifen (`While`) und bedingte Verzweigungen (`If`).
- **Standardbibliothek (StdLib)**: Optimierte Rust-Implementierungen für Mathematik, bitweise Operationen und Speicherverwaltung.

## Technischer Showcase: Voxel-Engine POC

Dieses Repository enthält eine Voxel-Engine als Proof-of-Concept. Dieser Showcase demonstriert die Leistungsfähigkeit von AetherCore:
- Persistente Zustandsverwaltung in großflächigen Voxel-Karten.
- Echtzeit-Physik und AABB-Kollisionserkennung.
- Fortschrittliche WGSL-Shader mit gerichteter Beleuchtung und Nebel-Effekten.
- Prozedurale Geländegenerierung mittels nativem Perlin-Rauschen.

---
**Designed for Machine Intelligence. Powered by Rust.**
