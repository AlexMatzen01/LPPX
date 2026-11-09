# LPPX — Legible Programming Platform eXtensions

**LPPX** is the extension platform for [LPPO](https://github.com/AlexMatzen01/lppo), the Legible Programming Paradigm Optimizer. LPPX expands LPPO through independently installable, versioned, Rust-powered extensions.

LPPX is **not** simply a graphics engine. It is a broader extension platform that supports graphics, libraries, frameworks, and tools. Any Rust/Cargo project that follows the LPPO package conventions can be distributed as an LPPX extension.

## What LPPX provides

- A package catalog (`~/.LPPO/packages.json`) describing available extensions.
- Independent installation, versioning, updating, and removal of each extension.
- Rust/Cargo builds coordinated by LPPO's integrated package manager (`lppo pkg`).
- Shared Rust crates so extensions reuse code without forcing users to install unrelated packages.

## Repository organization

Extensions live in independent subdirectories, each a separate Cargo project:

```text
LPPX/
├── LPPX2D/          # 2D graphics extension (independently installable)
├── LPPX3D/          # 3D graphics extension (independently installable)
├── common/          # shared Rust crate (math, windowing, input, timing)
├── docs/            # documentation
├── examples/        # example programs
├── Cargo.toml       # workspace manifest
├── README.md
├── LICENSE
└── .gitignore
```

Every independently installable extension has its own `Cargo.toml` and version. Installing LPPX2D does **not** install LPPX3D, and vice versa.

## The initial extensions

### LPPX2D — 2D graphics framework

A 2D graphics framework for LPPO applications and games:
- Window creation
- Render loop
- Basic shapes (rectangles, circles, lines, points)
- Colors
- Basic input
- Frame timing

Crate: `lppo-x2d` (`LPPX2D/`), version `1.0.0`.

### LPPX3D — 3D graphics framework

A 3D graphics framework for LPPO:
- Window creation
- 3D scene, camera, projections
- Meshes (including a built-in cube)
- Materials and colors
- Basic input

Crate: `lppo-x3d` (`LPPX3D/`), version `1.0.0`.

## How extensions relate to LPPO

LPPO ships with an integrated package manager:

```bash
# Search the package library
lppo pkg search LPPX2D

# Install an extension (resolves deps, builds with Cargo, tracks install)
lppo pkg install LPPX2D

# List installed extensions
lppo pkg list

# Remove an extension
lppo pkg remove LPPX2D

# Update all extensions
lppo pkg update

# Build without marking installed
lppo pkg build LPPX2D
```

The package library lives at `~/.LPPO/packages.json`. Installed packages are tracked separately at `~/.LPPO/installed.json`.

## Building from source

```bash
cd LPPX
cargo build --workspace      # build all extensions
cargo test --workspace       # run all extension tests
cargo build -p lppo-x2d      # build only LPPX2D
cargo build -p lppo-x3d      # build only LPPX3D
```

## Creating new LPPX extensions

1. Add a new directory (e.g. `MyExt/`) with its own `Cargo.toml`.
2. Make it a genuine Cargo library project (`crate-type = ["rlib", "cdylib"]`).
3. Add `lppo_interface.json` describing the LPPO-facing API.
4. (Optional) Add `lppo_package.json` with LPPO package metadata (dependencies, LPPO version).
5. Publish with `lppo pkg publish <path>`.

See each extension's `README.md` for details.

## Graphics compilation

LPPX extensions integrate with LPPO's compiler through a typed graphics intermediate representation. LPPO graphics programs are lowered into IR, verified, optimized, and lowered to native graphics code rather than being interpreted as raw library calls.

## License

MIT
