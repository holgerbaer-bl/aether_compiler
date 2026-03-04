# Changelog: KnotenCore Engine

**Vision:** A high-performance, general-purpose hybrid language (JIT/AOT) with native WGPU rendering and deterministic ARC memory management.
**Development Standard:** To ensure absolute version integrity, the architect must guarantee that every single sprint is cleanly pushed to the Git repository by the autonomous agent. This successful push must be explicitly documented in every sprint report.

---

## [v0.55.0] - Sprint 55: The UI Hardening Sprint
Resolved critical UI type inconsistencies and introduced native horizontal layout and fullscreen panel nodes.

### Fixed
- **`UIButton` Type Mismatch (#1):** `UIButton` now returns `RelType::Bool` instead of `RelType::Int`. Direct use as an `If`-condition now works natively without type coercion.
- **`RelType::Display` Annotation (#2):** `Display` now renders pure human-readable values (`42`, `true`, `hello`). Debugging output (with type tags) has been moved to the `Debug` trait. The internal `execute()` test harness uses `Debug` to keep test suite unbroken.
- **Egui Depth Buffer (#6):** Confirmed that the Egui 2D render pass uses `depth_view_opt` which resolves to `None` for 2D-only renderering. No Z-test on UI passes.
- **Windows EventLoop (#7):** Confirmed `with_any_thread(true)` fix already in place from Sprint 54.

### Added
- **`UIHorizontal(Box<Node>)`**: Renders child nodes side-by-side in a single egui horizontal layout row. Enables button grids, toolbars, and multi-column forms.
- **`UIFullscreen(Box<Node>)`**: Renders a borderless, zero-title-bar panel covering the entire canvas. Ideal for immersive game HUDs and full-screen overlay UIs.
- **`UISetStyle` Button Colors (#5):** Extended `UISetStyle` to 6 arguments with two optional trailing RGBA arrays for `button_idle` and `button_hover` colors. Backward-compatible: scripts using only 4 args still work.

---

## [v0.54.0] - Sprint 54: The Styling & Persistence Update
Introduced panic-resilient File I/O mappings and dynamic EGUI stylistic overrides powered natively by the JSON AST.

### Added
- **File I/O Persistence**: Engineered `registry_read_file` and `registry_write_file` using `std::fs` natively, with robust error catching to prevent runtime panics within the ARC registry.
- **Global UI Style Engine**: Bound a new AST node `UISetStyle` manipulating the global `egui::Context` rendering frame. Modifiable metrics include Window Rounding, Item Spacing, RGBA Accent coloring, and Background Panel shading, perfect for rendering Glassmorphism and Flat Design.
- **The Ultimate Constant**: Bound `registry_get_ultimate_answer` returning 42 natively via the FFI.
- **AOT & JIT Node Integration**: Upgraded the `executor.rs` stack and `optimizer.rs` counting arrays to safely recurse into all new stylistic nodes.

---

## [v0.53.0] - Sprint 53: The Kinetic Update (Input System)
Successfully implemented a universally applicable, thread-safe input handling system that bridges both game-engine inputs and software application inputs natively.

### Added
- **Global `InputState`:** Implemented as an `Arc<Mutex>` resolving all `DeviceEvent` and `WindowEvent` hooks from winit 0.30 via a new `pump_app_events` method on the main EventLoop.
- **Physical Keys (Gaming):** Maintained via `winit::keyboard::KeyCode` mapped in a `HashSet` for instantaneous queries over WASD / Arrow Keys.
- **Mouse Motion (3D/FPS):** Raw optical sensor deltas (`DeviceEvent::MouseMotion`) gracefully accumulate per-frame into `mouse_dx` and `mouse_dy`, completely untied from the UI cursor.
- **Text Typing (Software):** Automatically respects shift/caps keyboard contexts via `event.logical_key`, yielding the active `last_char` in exact u32 unicode form for native text-editing.
- **FFI & ARC Synchronization:** Added 4 new endpoints in `bridge.rs` (`registry_is_key_pressed`, `registry_get_mouse_delta_x/y`, `registry_get_last_char`), all casting safely to the `RelType` schema.

### Changed
- **Thread-Safe Resets:** Ensured exact VSync rendering intervals within `registry_window_update` before pumping the upcoming frame, protecting accumulation values during complex AST script loops.

---

## [v0.52.0] - Sprint 52: The 3D Hallway Flex
Extended the bare-metal WGPU integration from 2D billboarding/UI to true 3D spatial rendering. 

### Added
- **Camera Buffer:** Added a `wgpu::Buffer` and dedicated `BindGroup` for the Camera Uniform to feed the projection-view matrix.
- **Z-Buffer Depth Ordering:** Instantiated a `wgpu::TextureFormat::Depth32Float` texture attachment configured with `CompareFunction::Less` to correctly sort overlapping geometric quads.
- **GLAM Matrix Math:** Introduced `glam` dependency to dynamically assemble the camera `perspective_rh_gl()` and object `Mat4::from_scale_rotation_translation`.
- **PushConstants:** Upgraded pipeline layout to inject 64-byte `model_matrix` push constants per draw-call, avoiding dynamic uniform buffers.
- **Native AST Bindings:** Added `registry_set_camera` to mathematically orbit the scene camera, and refactored `registry_draw_quad_3d` to accept floating point coordinates (x, y, z, sx, sy).

---

## [v0.50.0] - Sprint 50: The Great ARC Reforging
Resolved critical memory vulnerabilities identified during the external security audit.

### Fixed
- **Core ARC Safety:** `registry_free` now safely wraps `registry_release` instead of removing handles, properly honoring the `ref_count`.
- **Panic Protection:** Fixed blind mutex locks (`unwrap()`), replacing them with `unwrap_or_else(|e| e.into_inner())` to prevent fatal panic poisoning.
- **RelType Clone-Bug:** `RelType` now properly manages its deep structure via a manual `Clone` implementation, guaranteeing that cloning inherently bumps its ARC `ref_count`.
- **AOT Memory Tracking:** The AOT transpiler now tracks `is_handle` block by block. Overwriting a native handle explicitly injects a `registry::registry_release()` natively into the compiled Rust output block, resolving all loop-based memory leaks.

---

## [v0.48.0] - Sprint 48: The Lexicon
Empowered KnotenCore with nested Key-Value dictionaries mapped to the standard Rust `HashMap`.

### Added
- **Native Maps:** Added `Type::Map` and corresponding AST Node variants (`MapCreate`, `MapSet`, `MapGet`, `MapHasKey`).
- **Deep ARC Integration:** JIT and AOT engines inherently support iterative de-allocation for maps. AOT intercepts assigned combinations utilizing Maps holding handles, iterating over inner keys to statically inject `registry_release` during scope exit.