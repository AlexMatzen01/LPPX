# Plan: LPPX2D — its own shading language (`.x2d`), compiled into LPPO projects

**Goal:** Make LPPX2D a real shading language. LPPO projects reference `.x2d` shader files from `.lppo` source; `lppo project build` compiles those shaders into the program's graphics IR at build time; the program renders them at runtime via a CPU renderer (default) with an optional GPU backend.

## Context (current state)

- `LPPX2D/` (`lppo-x2d` crate, v1.0.0) is a self-contained Rust/Cargo package in `/home/alex/Documents/LPPX`.
- Current LPPX2D: null-backend `Canvas`/`Window2d` with shapes (`rectangle`, `circle`, `line`, `point`), `Color`, `Vec2`, C-ABI FFI exports (`x2d_*`) called from LPPO via `libloading`. `lppo_interface.json` declares 6 functions; `lppo_package.json` declares package metadata.
- `LPPX/` workspace has `common/`, `LPPX3D/`, workspace `Cargo.toml`.
- LPPO (`/home/alex/Documents/lppo`): integrated package manager (`lppo pkg`), project manager (`lppo project {init,build,run,...}`), typed graphics IR (`src/pkg/graphics_ir.rs` with `GProgram`, `GOp`, `GBuilder`, `verify`), `lppo.toml` manifest, bundled-executable builds.
- IR already represents vec2/vec3/vec4/mat4/color/mesh/material/camera, `make_color`, `draw_rectangle`, `draw_circle`, `clear`, `begin_frame`, `end_frame`, SSA `GId`s, and verification.

## Decisions (final)

1. **Shading language:** A small text language in `.x2d` files. Declarative, fragment-shader-oriented: uniforms, `vec2`/`color` literals, shape primitives (`rect`, `circle`, `line`, `point`), a `main` block that emits draw commands. Not GLSL; purpose-built for LPPX2D.
2. **`.lppo` → `.x2d` reference:** `use LPPX2D "assets/shader.x2d";` — the string is the shader path relative to the `.lppo` file's directory. Resolved at build time.
3. **Build-time compilation:** `lppo project build` scans the entry `.lppo` and transitively-resolved imports for `use LPPX2D "..."`, compiles each `.x2d` via the LPPX2D shader compiler into a `GProgram`, and bundles the resulting IR (serialized) into the executable.
4. **Rendering backend:** CPU software renderer is the default (works everywhere, no GPU dependency). GPU backend (wgpu) is an optional crate feature `gpu`, off by default. LPPX2D ships both; the runtime backend is selected by a `render_backend` field in `lppo.toml` / a runtime call, defaulting to CPU.
5. **Ownership:** LPPX2D remains independently installable (`lppo pkg install LPPX2D`) and versioned separately from LPPX3D. It may depend on `lppo-common` only for shared math; the shader compiler itself is self-contained.
6. **FFI:** The existing `x2d_*` C-ABI functions remain for direct calls; the new shading-language path compiles shaders to IR and executes them through the existing graphics-IR runtime path, not new FFI per draw call.

## Plan file location
`/home/alex/Documents/lppx/.kilo/plans/1794257295906-lppx2d-shading-language.md`

## Tasks

### 1. Define the `.x2d` shading language syntax & semantics
File: `docs/x2d-language.md` (new) + implement parser.

Syntax (proposal):
```
uniform color u_fill = rgba(1, 0, 0, 1);
uniform vec2 u_size = vec2(200, 150);

shader rect_demo {
    clear rgba(0, 0, 0, 1);
    rect 100, 100, 200, 150 fill u_fill;
    circle 300, 200, 50 fill rgba(0, 0, 1, 1);
}
```
Semantics:
- Top-level `uniform <type> <name> = <literal>;` declarations (types: `float`, `vec2`, `color`, `int`).
- `shader <name> { ... }` blocks containing draw commands and a `clear`.
- Commands: `rect x, y, w, h fill <color>;` / `circle cx, cy, r fill <color>;` / `line x1, y1, x2, y2 color <color>;` / `point x, y color <color>;` / `clear <color>;`.
- Literals: `rgba(r,g,b,a)`, `vec2(x,y)`, numbers.
- A `.x2d` file may contain multiple `shader` blocks; each becomes one `GProgram`.

Deliverable: `LPPX2D/src/shader_lang/{mod.rs, lexer.rs, parser.rs, ast.rs, lower.rs}` — parse `.x2d` text → `GProgram` (reuse `crate::graphics_ir` types via a `graphics_ir` dependency or copy the needed IR types into LPPX2D to avoid circularity). Recommendation: LPPX2D depends on `lppo-common` only; for IR types, either re-export a small `graphics_ir` crate from LPPO's workspace or duplicate the minimal `GOp`/`GProgram`/`GType` enum in LPPX2D with serde so it can be serialized into the LPPO bundle. To keep LPPX2D independently installable, duplicate the minimal IR enum inside LPPX2D (`src/ir.rs`) and serialize with serde; LPPO deserializes it at build/runtime.

Wait — LPPO already has `graphics_ir`. LPPX2D is a separate repo/crate. To avoid LPPX2D depending on LPPO (which would break independent installability), LPPX2D should define its own IR enum (mirroring `GOp`) and serialize it. LPPO's `lppo project build` will deserialize LPPX2D's IR bytes and merge them into the program's graphics IR. This keeps LPPX2D standalone.

### 2. LPPX2D crate changes
- Add `src/ir.rs`: serializable `GProgram`/`GOp`/`GType`/`GOperand` mirroring LPPO's graphics IR (serde).
- Add `src/shader_lang/`: lexer, parser, AST, lowerer producing `ir::GProgram`.
- Add `src/compiler.rs`: `compile_x2d(source: &str) -> Result<Vec<u8>, ...>` serializing the `GProgram` to JSON/bincode bytes.
- Add backends:
  - `src/backend/cpu.rs`: software renderer that executes a `GProgram` (draws to a pixel buffer / prints). Default feature.
  - `src/backend/gpu.rs` (feature `gpu`): wgpu-based renderer (optional, behind feature flag).
  - `src/backend/mod.rs`: `Backend` trait `render(program: &GProgram, canvas: &mut Framebuffer)`.
- Update `Cargo.toml`: add `serde`, optional `wgpu` behind `gpu` feature; keep `crate-type = ["rlib","cdylib"]`.
- Keep existing `x2d_*` C-ABI FFI exports (unchanged, still work).
- Add `lppo_interface.json` entries for new shader functions if any are exposed directly (e.g. `x2d_shader_run(program_ptr, len)`). Optional; the build-time path may not need new FFI if IR is embedded in the bundle. Mark as future/not-in-scope for v1.
- Tests: parser round-trip, lower `.x2d` → IR, verify IR, CPU backend renders without panic.

### 3. LPPO project build integration (`lppo project build`)
- In `project.rs` `cmd_build`: after reading entry source, walk `use LPPX2D "..."` declarations (and transitively through imported `.lppo` files), resolve each `.x2d` path relative to the referencing `.lppo` file.
- For each `.x2d`: invoke LPPX2D's compiler (either by linking the `lppo-x2d` Rust lib directly if available, or by shelling out to a small LPPX2D compiler binary, or by embedding a copy of the shader compiler). Simplest robust approach: `lppo project build` shells out to `lppo-x2d` compiler via `lppo pkg`-installed LPPX2D? No — build-time must not require a separate install. Better: vendor the LPPX2D shader compiler into LPPO as a dev/build dependency, OR add a `lppo project build --x2d` that requires LPPX2D source on `CARGO_PATH`. 
  Decision: LPPO's `lppo project build` calls into an LPPO-internal `x2d_compiler` module that is a *copy* (or re-export) of the LPPX2D shader compiler logic, so builds are self-contained. To avoid duplication, LPPO can depend on `lppo-x2d` via path only when building projects — but that couples LPPO to LPPX2D. 
  Cleanest: introduce an LPPO feature `x2d` that, when enabled, depends on `lppo-x2d` by path (`../LPPX/LPPX2D`) and exposes `compile_x2d_shader`. `lppo project build` with the `x2d` feature compiles shaders; without it, `.x2d` references are an error unless precompiled `.x2d.json` sidecars exist. Keep scope: implement the `x2d` feature + path dependency for the build tool; document that the LPPX repo must be adjacent for shader compilation.
- Bundle compiled shader IR bytes into the executable payload (extend the `LPPO-EMBED-1` bundle format or add a second marker section for shader IRs keyed by name).
- Runtime: `Executor` loads shader IRs from the bundle and exposes them; `use LPPX2D "shader.x2d";` at runtime looks up the compiled shader by name and executes it via the CPU backend.

### 4. LPPO runtime integration
- `Executor` gains `shader_programs: HashMap<String, GProgram>` loaded from bundle.
- `eval_expr` for `use LPPX2D "..."` no longer resolves on disk (already skipped for installed packages); instead it binds the precompiled shader IR. A new runtime op or builtin `x2d_run(<shader_name>)` executes the shader via the CPU backend.
- CPU backend must be linkable into LPPO (vendor the renderer into LPPO's `x2d` feature or call out to LPPX2D cdylib). Decision: vendor a minimal CPU renderer in LPPO under `pkg/x2d_cpu.rs` that interprets the shared IR enum. This keeps LPPO self-contained.

### 5. Examples
- `LPPX/examples/shader_demo.lppo` + `LPPX/examples/shader_demo.x2d` — draws a rectangle and circle using the shading language.
- `LPPX/LPPX2D/examples/` — standalone `.x2d` + Rust test harness.
- README updates for LPPX2D shading language.

### 6. Tests
- `LPPX2D`: shader parse + lower + verify round-trip tests; CPU backend render test (no panic, produces framebuffer).
- LPPO: `lppo project build` on a project with a `.x2d` reference produces an executable that runs and renders the shader (CPU backend).
- Independence: `lppo pkg install LPPX2D` does not install LPPX3D; LPPX2D builds standalone with `cargo build -p lppo-x2d` (no features) and with `--features gpu`.

## Risks / open items
- IR type parity between LPPX2D's serialized IR and LPPO's `GProgram` must be maintained; document the schema.
- wgpu GPU backend is optional and not guaranteed to run in headless/CI environments; CPU backend is the gated, testable path.
- `lppo project build` shader compilation requires the LPPX2D compiler; document the path dependency / feature requirement.
- `lppo.toml` needs a `[graphics]` or `[x2d]` section to select backend; keep default CPU so nothing breaks.

## Validation
- `cargo build -p lppo-x2d` (default, CPU-only) succeeds.
- `cargo test -p lppo-x2d` passes (parser/lower/verify/CPU backend).
- Create a scratch LPPO project with `src/main.lppo` referencing `shader.x2d`; `lppo project build` produces an executable; running it prints/renders the shader output via CPU backend; no LPPX3D installed.
- `lppo pkg install LPPX2D` into a project's `.lppo/` store works and is independent of LPPX3D.
