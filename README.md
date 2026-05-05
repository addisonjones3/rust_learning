# rust_learning

A Rust coding exercise solved and built with [Bazel](https://bazel.build/), using [`rules_rust`](https://github.com/bazelbuild/rules_rust).

The exercise involves implementing a span tracking system modeled after distributed tracing concepts — spans with lifecycle state, metadata tagging, and aggregate counters. The primary goal is not the exercise itself but demonstrating how to structure a Rust project under Bazel with `rules_rust`.

## Project structure

```
├── MODULE.bazel          # Bzlmod dependencies (rules_rust, gazelle, etc.)
├── Cargo.toml            # Cargo workspace (used by gazelle_rust for BUILD generation)
├── lib_questions/        # Shared library crate with the span implementation
│   ├── src/
│   │   ├── lib.rs
│   │   └── span.rs
│   └── BUILD.bazel
└── questions/obs/        # Binary crate — reads input and drives the span system
    ├── src/main.rs
    └── BUILD.bazel
```

## Key tooling

| Tool | Role |
|------|------|
| `rules_rust` 0.68.1 | Rust toolchain and build rules inside Bazel |
| `gazelle_rust` | Auto-generates `BUILD.bazel` files from `Cargo.toml` manifests |
| `crate_universe` | Resolves and vendors external crate dependencies via `Cargo.lock` |
| Rust 1.92.0 (edition 2024) | Language version pinned in `MODULE.bazel` |
| Bazel 8.6.0 | Build system version pinned in `.bazelversion` |

Clippy runs automatically on every build via a Bazel aspect (`.bazelrc`).

## Requirements

- [Bazelisk](https://github.com/bazelbuild/bazelisk) (reads `.bazelversion` automatically)
- No separate Rust toolchain install required — `rules_rust` downloads one

## Build & run

```bash
# Build everything
bazel build //...

# Run the observability binary (reads from stdin)
bazel run //questions/obs

# Run tests
bazel test //...

# Regenerate BUILD files from Cargo.toml (via gazelle_rust)
just g

# Run first-party cargo commands using the Bazel wrapper
just cargo check
```

A `justfile` is included with aliases for common Bazel-wrapped Cargo commands (`just cargo`, `just cargo-clippy`, `just gazelle`, etc.).
