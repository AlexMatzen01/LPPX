# Graphics Compilation in LPPX

## Architecture

LPPX graphics extensions integrate with LPPO's compiler through a **typed
graphics intermediate representation (IR)**. This is a real lowering path:
LPPO graphics operations are lowered into IR, verified, optimized, and then
lowered to native graphics code — they are not merely interpreted as runtime
library calls.

## MLIR / Rust MIR evaluation

MLIR and Rust MIR were evaluated as candidate host-program IRs:

- **MLIR**: powerful and modular, but integrating LLVM/MLIR into the LPPO
  compiler requires substantial tooling and a stable C/Rust binding story.
- **Rust MIR**: an internal `rustc` representation with no stability guarantees;
  it is not a public embedding target and generating Rust source does not
  constitute direct MIR support.

Neither was practical to integrate responsibly at this stage, so LPPX uses a
**typed, verifiable internal graphics IR** implemented in LPPO
(`src/pkg/graphics_ir.rs`), with a documented future lowering path to MLIR or a
native graphics backend.

## Graphics IR

The IR explicitly represents:
- 2D/3D vectors, points, matrices, transforms
- colors, vertices, indices, meshes, geometry
- textures, samplers, materials, shaders
- cameras, projections, render targets, render passes
- draw operations, GPU resource ownership and lifetimes
- window/surface creation, input, frame timing
- typed numeric operations and errors

Each operation carries a declared type (`float`, `int`, `vec2`, `vec3`, `vec4`,
`mat4`, `color`, `texture`, `mesh`, `material`, `camera`, ...). The IR is in
SSA form: every `Ref` operand must refer to a `GId` defined earlier in the
program.

## Verification

`graphics_ir::verify(program)` checks:
- every referenced `GId` is defined earlier (SSA dominance);
- every operand's type matches the operation's expected operand types;
- no undefined identifiers.

Verification errors report the operation index, operation name, and a message
linked to the IR construct.

## Lowering

`GBuilder` constructs a `GProgram` while assigning fresh `GId`s, providing a
lowering boundary from higher-level LPPO graphics constructs to IR. Example:

```rust
let mut b = GBuilder::new();
let color = b.make_color(GBuilder::const_f(1.0), GBuilder::const_f(0.0),
                         GBuilder::const_f(0.0), GBuilder::const_f(1.0));
b.draw_rectangle(GBuilder::const_f(0.0), GBuilder::const_f(0.0),
                 GBuilder::const_f(10.0), GBuilder::const_f(10.0),
                 GOperand::Ref(color));
assert!(verify(&b.program).is_ok());
```

## Backend abstraction

The compiler separates:
- IR construction and verification
- native code generation
- graphics backend (windowing, GPU resources, shaders)
- LPPO language semantics

Shaders (where needed) use a separate compilation pipeline (e.g. SPIR-V) and
are not confused with the host program's IR pipeline.

## Future work

Lower the LPPO graphics IR into MLIR (custom graphics dialect) or directly into
a native graphics backend (e.g. wgpu), then emit SPIR-V shaders and native
executable code. The IR is deliberately typed and verifiable so that path is
well-defined.
