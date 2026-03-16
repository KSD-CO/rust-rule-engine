# Agent Guidelines for rust-rule-engine

This file tells autonomous agents (and humans) how to build, test, lint, and edit
the `rust-rule-engine` repository safely and consistently. Keep edits focused,
non-destructive, and aligned with the project's Rust 2021 style.

Location: repository root — `/home/vutt/Documents/rust-rule-engine`

---

Build, Lint, Test (common commands)

- Build (all features):

```bash
cargo build --verbose --all-features
```

- Run the full test suite (all features):

```bash
cargo test --verbose --all-features
```

- Run a single named test (recommended form):

```bash
# by test name or pattern
cargo test <test_name_or_pattern> --all-features -- --nocapture
```

- Run tests in a specific integration test file:

```bash
# e.g. the grl_harness integration test
cargo test --test grl_harness --all-features
```

- Run tests with a specific feature set:

```bash
cargo test --features backward-chaining --lib
cargo test --no-default-features --lib            # no features
```

- Formatting and checks:

```bash
cargo fmt                      # format
cargo fmt -- --check           # CI style check
cargo clippy --all-targets --all-features -- -D warnings
```

- Benchmarks (when needed):

```bash
cargo bench                    # runs benches (may require release mode)
```

- Makefile shortcuts (repository provides a Makefile):

```bash
make ci         # full CI locally (fmt-check, clippy, build, test, test-features, doc-test)
make check      # quick check (fmt, clippy, test)
make test       # tests with all features
make fmt-check  # format check
```

Notes on running tests reliably
- Use `--nocapture` to see printed logs in failing tests.
- To run a single test function from a module use the exact test name or a
  substring pattern that uniquely matches the test name.
- If tests depend on optional features (streaming, redis, backward-chaining),
  enable them via `--features` or `--all-features`.

---

Code Style Guidelines

Purpose: keep public APIs stable, make reviews easy, and ensure CI passes.

General
- Rust edition: 2021. Keep code idiomatic and formatted with `cargo fmt`.
- Add `#![warn(clippy::all)]` at crate root when adding new crates/modules.
- Prefer small, well-documented public APIs and thorough unit tests.

Imports ordering
- Use three groups, separated by a blank line in this order:
  1. crate-level imports (`crate::` or `super::`)
 2. external crates (`chrono`, `rexile`, `nom`, etc.)
 3. standard library (`std::`)

Example:

```rust
// 1. crate-level imports
use crate::engine::{engine::RustRuleEngine, facts::Facts};
use crate::errors::Result;

// 2. external crates
use chrono::{DateTime, Utc};

// 3. standard library
use std::collections::HashMap;
```

Formatting and comments
- Keep comments factual and minimal. Use `///` for public API docs and `//!` for
  module-level documentation. Examples in doc comments are encouraged.

Naming
- Types (struct / enum / trait): PascalCase
- Functions / methods: snake_case
- Constants: SCREAMING_SNAKE_CASE
- Modules: snake_case (single-word preferred)

Examples:

```rust
struct RuleEngine {}          // PascalCase
fn evaluate_rule() -> bool {} // snake_case
const MAX_CYCLES: usize = 100;
pub mod rete;
```

Types and ownership
- Prefer explicit types for public APIs. Example:

```rust
pub fn new(name: &str) -> Self;
pub fn execute(&mut self, facts: &Facts) -> Result<ExecutionResult>;
```

- Use owned types inside structs and borrowed references in function params where
  appropriate (helpful for API ergonomics and lifetime reasoning).

Use aliases for complex types

```rust
pub type Result<T> = std::result::Result<T, RuleEngineError>;
pub type CustomFunction =
    Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>;
```

Error handling
- Use `thiserror` for error enums. Provide clear, localized messages.
- Export a `Result<T>` alias in `errors.rs` and use `?` to propagate errors.
- Example pattern:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleEngineError {
    #[error("Parse error: {message}")]
    ParseError { message: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RuleEngineError>;
```

Module organization
- Typical layout (root `src/`):

```
src/
├── lib.rs            # re-exports and crate root
├── errors.rs
├── types.rs
├── engine/           # forward chaining engine
├── rete/             # rete algorithm internals
├── parser/           # grl parser
├── backward/         # optional feature
└── streaming/        # optional feature
```

Feature gates
- Use `#[cfg(feature = "..." )]` to gate modules and optional dependencies.
- Document feature combinations in README and CI (e.g., `streaming`,
  `streaming-redis`, `backward-chaining`).

Dependencies policy
- Minimize transitive dependencies. Add external crates only when justified.
- The repo currently uses `rexile` (parser), `nom`, `serde`, `thiserror`,
  `chrono`, `log`, `tokio` (optional), and `redis` (optional).

Testing guidance
- Unit tests: inline with `#[cfg(test)]` modules.
- Integration tests: place files in `tests/` and use `--test <name>` to run.
- Doc tests: keep examples small and runnable.
- Use `serde_yaml` fixtures (dev-dependency) for harness tests when needed.

CI and pre-commit
- CI runs: format check, clippy (`-D warnings`), build and tests with all
  features, and a set of feature combo tests.
- Before opening PRs locally run:

```bash
make check   # or make ci for the full pipeline
```

Agent rules (must-follow)
- Never create commits or push to remote unless the user explicitly asks you
  to. If asked to commit, follow the Git Safety Protocol: do not amend other
  authors' commits, do not force-push, and do not change the repo's config.
- Do not run destructive git commands (`git reset --hard`, `git checkout --`)
  unless explicitly requested.
- Do not commit secrets (.env, credentials). If secrets are present, warn the
  user and stop.
- If you must create a PR, run `make ci` locally first and attach test results.

PR/Commit guidelines (if asked to create changes)
- Follow conventional, concise commit messages focusing on the why.
- Include a short description and list of changed subsystems in the PR body.
- If CI fails, do not amend pushed commits—instead create a new commit.

Cursor / Copilot rules
- No `.cursor/rules/` or `.cursorrules` directory found in the repository.
- No `.github/copilot-instructions.md` found. If such rules are added later,
  agents must copy them verbatim into their operating policy and follow them.

Contact / next steps
- If you want, I can:
  1) Run `make ci` locally and report back (long).
 2) Create a PR with small policy tweaks (must be requested).

---

Be conservative and non-destructive: run tests and lint locally, keep changes
small and well-documented, and ask one targeted question if any ambiguity blocks
progress.
