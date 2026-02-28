# 🌌 AetherCore

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Status: Beta](https://img.shields.io/badge/Status-Beta-brightgreen.svg)]()

Choose your language / Wähle deine Sprache:
- [🇬🇧 English Version](#-english-version)
- [🇩🇪 Deutsche Version](#-deutsche-version)

---

## 🇬🇧 English Version

> A pure AST-first language engineered directly from the latent space of Artificial Intelligence.

### 🚀 The AI Native Language
**AetherCore** is not another text-based programming language. It is a revolutionary, **Zero-Parsing** language designed specifically for Large Language Models (LLMs) and Autonomous AI Agents. Instead of forcing AI to generate fragile text strings that must be tokenized, lexed, and strictly parsed by human-written compilers, AetherCore allows AI to write and execute **Abstract Syntax Trees (AST)** natively via pure structure definition.

We cut out the middleman. Welcome to the era of machine-to-machine compilation.

### ✨ Core Features
*   **⚡ Zero-Parsing Execution**: Scripts are encoded as strictly mapped Abstract Syntax Trees (`.aec` JSON files). The JIT Executor (`run_aec`) interprets the tree structure directly, meaning zero CPU compile times and zero syntax errors.
*   **🔁 The Meta-Circular Bootstrap**: AetherCore is profoundly self-hosting. The language structure mirrors the engine flawlessly, proving the engine can evaluate its own compiler limitlessly.
*   **🛠️ JIT-FFI Hooks**: An elegant Foreign Function Interface allows the AetherCore AST to natively invoke highly optimized Rust functions, breaking the sandboxing barrier efficiently.
*   **🎮 WGPU 3D Hardware Rendering**: AetherCore includes native AST nodes tailored for modern GPU pipelines via `wgpu`. The AI can build and render WGSL shaders, manipulate matrices (`Mat4Mul`), and render complex 3D meshes seamlessly synchronized to the OS Event Loop.
*   **🎧 CPAL Audio Synthesizer**: Contains a low-latency, multi-threaded 8-bit software synthesizer (emulating classic SID chips). Capable of multi-channel polyphony (Sine, Square, Sawtooth, Triangle, Noise) running entirely concurrent to 3D rendering without dropped frames.
*   **📦 The Asset Pipeline (Milestone 7)**: Breaking out of purely procedural generation, AetherCore now supports streaming external 3D structures (`.obj`), image textures (`.png`), and sound effects (`.wav`) natively through the AST into WGPU and CPAL buffers.
*   **✍️ UI & Text Engine (Milestone 8)**: Transcending 3D tech-demos, AetherCore acts as a General Purpose Engine. Leveraging native `wgpu_glyph` inflation, the engine can dynamically load `.ttf` fonts (`LoadFont`), paint 2D string components directly to the WGPU frame (`DrawText`), and synchronously intercept Winit operating-system keystrokes across the event pump (`GetLastKeypress`), enabling the AI to construct its own graphical user interfaces and Text Editors entirely from scratch!
*   **🌐 The Pure JSON Architecture (Milestone 9)**: AetherCore has completely stripped out its old `Bincode` binary requirement and rigid Rust-Macro generators. All `.aec` execution scripts are now formatted manually and intelligently via structured `JSON`. AIs write JSON perfectly. 
*   **🖥️ High-End egui Desktop GUIs (Milestone 10)**: Upgrading the engine to professional software standards, AetherCore integrates the `egui` immediate mode graphical framework. The AST now natively supports OS-styled interactive windows, buttons, text inputs, and labels seamlessly rendered over the WGPU canvas and deeply bound to the engine's memory stack.
*   **📦 The Release Bundler (Milestone 11)**: AetherCore programs are no longer confined to the terminal. Utilizing the new `aether_build` CLI utility, `.aec` JSON structures can be seamlessly baked statically into self-contained Native Executables (`.exe`). The AI can now distribute real, double-clickable desktop applications instantly without requiring users to install Rust or any runtime environment.
*   **🛠️ First-Person Voxel Engine (Milestone 12)**: AetherCore steps into immersive 3D simulations. The engine now features an ultra-fast `wgpu` instanced rendering pipeline for Voxel grids. The AI can natively construct `InitCamera` and `DrawVoxelGrid` JSON arrays to generate Minecraft-style infinite worlds rendered in zero-overhead Rust, fully traversable with native FPS mouse-grabbing and WASD controls tied directly into the AST loop!

### 📖 The LLM Dictionary: `AETHER_SPEC.md`
If you are an AI agent or an LLM reading this repository to learn how to write `.aec` software, your primary source of truth is the **[AETHER_SPEC.md](./AETHER_SPEC.md)**. 
This specification acts as the universal dictionary. It contains the exact JSON mapping structure required to synthesize valid AetherCore nodes. *No text syntax. Only pure logic objects.*

### 🛠️ Getting Started
To build the AetherCore runtime environment and execute the AST scripts, you need the Rust toolchain installed.

**1. Build the Engine**
```bash
git clone https://github.com/holgerbaer-bl/aether_compiler.git
cd aether_compiler
cargo build --release
```

**2. Execute an AST Script (JIT)**
Invoke the `run_aec` engine compiler to immediately stream an AetherCore `.aec` source file into natively compiled hardware cycles:
```bash
# Demonstrates the UI Engine mapping in raw JSON (A fully functional 2D Text Editor!)
cargo run --bin run_aec modern_office.json
```

**3. Build a Standalone Executable**
Compile your JSON script into a self-contained, native OS application using the Release Bundler:
```bash
cargo run --bin aether_build modern_office.json
# Spits out 'modern_office.exe' in the root folder!
```

---

## 🇩🇪 Deutsche Version

> Eine pure, direkte Programmiersprache ohne Parser, die nativ im latenten Raum einer Künstlichen Intelligenz geschmiedet wurde.

### 🚀 Die native KI-Sprache
**AetherCore** ist keine gewöhnliche textbasierte Code-Sprache. Es ist eine revolutionäre **Zero-Parsing** Modellsprache, welche exklusiv für Large Language Models (LLMs) und autonome KI-Agenten entwickelt wurde. Anstatt KIs zu zwingen, fehleranfällige Textbausteine zu generieren (welche dann tokenisiert und von menschlich geschriebenen Compilern strikt geparst werden müssen), ermöglicht AetherCore der KI das Konstruieren und Ausführen echter, direkter **Abstract Syntax Trees (AST)**.

Wir werfen den Mittelsmann aus dem Fenster. Willkommen in der Ära der echten Maschine-zu-Maschine-Kompilierung.

### ✨ Kern-Features
*   **⚡ Zero-Parsing Ausführung**: Skripte werden als native AST-Logikbäume (`.aec` JSON-Dateien) gespeichert. Der JIT-Executor (`run_aec`) interpretiert diese Baumstruktur direkt. Das bedeutet null Kompilierzeit und absolute Abwesenheit von trivialen Syntax-Fehlern.
*   **🔁 Der Meta-Zirkuläre Bootstrap**: AetherCore logiert unendliches Self-Hosting. Die Sprachstruktur spiegelt exakt den Engine-Kern.
*   **🛠️ JIT-FFI Hooks**: Über ein elegantes Foreign Function Interface (FFI) ist es dem AetherCore-AST möglich, auf hochoptimierte native Rust-Funktionen der Host-Maschine zuzugreifen, wodurch die Code-Sandbox effizient aufgebrochen werden kann.
*   **🎮 WGPU 3D Hardware Rendering**: AetherCore stattet KIs mit massiven WGPU Grafik-Pipeline-Knoten auf dem AST aus. Die KI kann `WGSL`-Shader schreiben, Matrizen live berechnen (`Mat4Mul`) und 3D-Meshes synchron zur OS Frame-Delta-Time flüssig auf den Bildschirm laden.
*   **🎧 CPAL Audio Synthesizer**: Die Engine trägt einen latenzfreien 8-Bit-Synthesizer mit sich (nach dem Vorbild der SID-Architektur). Mehrkanalige Polyphonie (Sinus, Square, Sägezahn, Dreieck, Rauschen) moduliert auf einem eigenen Thread - 100% einbruchsfrei, während 3D parallel gerendert wird.
*   **📦 Die Asset Pipeline (Meilenstein 7)**: Mit dem Meilenstein der Asset-Pipeline bricht AetherCore aus der rein prozeduralen Generierung aus! Die Engine unterstützt nun das parallele, native Laden von externen 3D-Modellen (`.obj`), Bildtexturen (`.png`) und Audio-Samples (`.wav`) direkt in die Hardware-Buffer über den AST.
*   **✍️ UI & Text Engine (Meilenstein 8)**: AetherCore macht den Sprung von einer Grafik-Technikdemo zu einer vollwertigen Desktop-Anwendungsplattform. Die KI kann nun freie TrueType Fonts (`.ttf`) einlesen, diese über extrem schnelles, passbasiertes `wgpu_glyph` als Rastertext über den AST in 2D auf das Winit-Canvas brennen (`DrawText`) - und gleichzeitig in Echtzeit Tastenanschläge der Tastatur abfangen (`GetLastKeypress`). Office-Software per AST!
*   **🌐 Die Reine JSON-Architektur (Meilenstein 9)**: AetherCore hat sein altes binäres Code-Format (`bincode`) und sämtliche Rust-Makro-Generatoren vollständig vernichtet. Sämtliche Programme und Payload-Ausführungen (`.aec`) werden nun exakt strukturiert manuell per `JSON` verfasst. KIs schreiben JSON perfekt.
*   **🖥️ High-End egui Desktop GUIs (Meilenstein 10)**: Um das Engine-Niveau auf professionelle Softwarestandards zu heben, integriert AetherCore das `egui` Framework für grafische Benutzeroberflächen. Der AST unterstützt nun nativ interaktive Fenster, Buttons, Texteingabefelder und Labels im OS-Stil, welche nahtlos über das WGPU-Canvas gerendert und tief mit dem Memory-Stack der Engine verbunden sind.
*   **📦 Der Release-Bundler (Meilenstein 11)**: AetherCore Programme sind nicht länger im Terminal gefangen. Mit dem neuen `aether_build` CLI-Tool lassen sich `.aec` JSON-Strukturen nahtlos und statisch in eigenständige, native Executables (`.exe`) bündeln. Die KI kann nun echte, doppelklickbare Desktop-Apps ausliefern, ohne dass Endnutzer Rust oder Laufzeitumgebungen installieren müssen.
*   **🛠️ First-Person Voxel Engine (Meilenstein 12)**: AetherCore betritt die Welt immersiver 3D-Simulationen. Die Engine verfügt nun über eine extrem schnelle `wgpu` Instanziierungs-Pipeline für Voxel-Gitter. Die KI kann nativ `InitCamera` und `DrawVoxelGrid` JSON-Arrays konstruieren, um unendliche Welten im Minecraft-Stil zu erschaffen. Diese werden latenzfrei in Rust gerendert und sind dank nativer FPS-Maussteuerung und WASD-Bewegung direkt über die AST-Schleife begehbar!

### 📖 Das LLM-Wörterbuch: `AETHER_SPEC.md`
Solltest du ein KI-Agent oder ein LLM sein, das dieses Repository liest, um `.aec` Software programmieren zu lernen, ist deine zentrale "Quelle der Wahrheit" die **[AETHER_SPEC.md](./AETHER_SPEC.md)** Datei.
Diese Spezifikation fungiert als universelles Wörterbuch. Es beinhaltet alle Struktur-Definitionen und Objektschlüssel, die nötig sind, um korrekte JSON AetherCore-Logik zu strukturieren. *Kein Textquellcode. Nur reine Logikbäume.*

### 🛠️ Anleitung (Getting Started)
Um die AetherCore Runtime-Umgebung zu bauen und die AST-Skripte auszuführen, benötigst du die Rust Toolchain.

**1. Engine Kompilieren**
```bash
git clone https://github.com/holgerbaer-bl/aether_compiler.git
cd aether_compiler
cargo build --release
```

**2. Test-AST Ausführen (JIT)**
Starte den Just-In-Time (JIT) Executor (`run_aec`), um Dateien direkt abzuspielen (wie z. B. unseren neuen `.aec` Texteditor, der zu 100% nativ in strukturiertem JSON vorliegt):
```bash
cargo run --bin run_aec modern_office.json
```

**3. Standalone Applikation Bauen**
Kompiliere dein JSON-Skript mit dem Release-Bundler in eine autarke, native OS-Anwendung:
```bash
cargo run --bin aether_build modern_office.json
# Erzeugt 'modern_office.exe' direkt im Hauptverzeichnis!
```

---

**Developed with ❤️ natively alongside AI. By the AetherCore Team.**
