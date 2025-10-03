# 🚀 Publishing Guide for rust-rule-engine

## Before Publishing

### 1. Update Cargo.toml
Edit the following fields in `Cargo.toml`:
```toml
authors = ["James Vu <ttvuhm@gmail.com>"]
```

### 2. Create crates.io Account
- Visit https://crates.io/
- Sign in with GitHub
- Get your API token from https://crates.io/me

### 3. Login to cargo
```bash
cargo login
# Enter your API token when prompted
```

## Publishing Process

### Quick Publish (Recommended)
```bash
# Run all checks and publish
make release
```

### Step by Step

#### 1. Quality Assurance
```bash
make format    # Format code
make lint      # Run clippy
make test      # Run tests
make examples  # Test all examples
```

#### 2. Git Management
```bash
make git-commit  # Commit all changes
```

#### 3. Test Publishing
```bash
make publish-dry  # Dry run (test only)
```

#### 4. Real Publishing
```bash
make publish-real  # Actual publish
```

## Version Management

### Bump Version
```bash
make version-patch  # 0.1.0 -> 0.1.1
make version-minor  # 0.1.1 -> 0.2.0
make version-major  # 0.2.0 -> 1.0.0
```

## Useful Commands

### Development
```bash
make build         # Build project
make check         # Quick check
make docs          # Generate docs
make clean         # Clean build files
```

### Package Info
```bash
make package-info  # Show package details
```

### Quality Assurance
```bash
make qa           # Run all quality checks
```

## Troubleshooting

### Uncommitted Changes
If you get an error about uncommitted changes:
```bash
make git-commit   # Commit changes
# OR
cargo publish --allow-dirty  # Publish with uncommitted changes
```

### Missing API Token
```bash
cargo login
# Enter your token from https://crates.io/me
```

### Failed Examples
If examples fail, check:
- Rule files in `examples/rules/` exist
- Custom functions are properly registered
- Fact data matches rule conditions

## Publishing Checklist

- [ ] Update author email in Cargo.toml
- [ ] Update repository URL in Cargo.toml
- [ ] Set up crates.io account and API token
- [ ] Run `make qa` successfully
- [ ] Run `make examples` successfully
- [ ] Run `make publish-dry` successfully
- [ ] Commit all changes
- [ ] Run `make publish-real`

## Post-Publishing

After successful publishing:
1. Check your crate at https://crates.io/crates/rust-rule-engine
2. Verify documentation at https://docs.rs/rust-rule-engine
3. Test installation: `cargo add rust-rule-engine`

## Example Usage After Publishing

```toml
# Cargo.toml
[dependencies]
rust-rule-engine = "0.1.2"
```

```rust
use rust_rule_engine::engine::{RustRuleEngine, EngineConfig};
use rust_rule_engine::parser::grl_parser::GRLParser;

// Your code here...
```
