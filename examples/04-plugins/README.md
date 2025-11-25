# Plugin System Examples

Plugin system for Rust Rule Engine - extending functionality.

## Example List

- **plugin_system_demo.rs** - Basic plugin system
- **builtin_plugins_demo.rs** - Built-in plugins
- **advanced_plugins_showcase.rs** - Advanced plugin showcase

## Plugin Implementations

The `plugins/` directory contains plugin implementations:

- **string_utils_plugin.rs** - String manipulation utilities
- **database_plugin.rs** - Database integration
- **http_client_plugin.rs** - HTTP client for external API calls
- **notification_plugin.rs** - Notification system
- **aiml_plugin.rs** - AI/ML integration

## How to Create a Plugin

1. Implement `Plugin` trait
2. Register plugin with engine
3. Use in rules

Example:
```rust
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn execute(&self, context: &mut Context) -> Result<()> {
        // Plugin logic
        Ok(())
    }
}
```

## How to run

```bash
cargo run --example plugin_system_demo
cargo run --example builtin_plugins_demo
cargo run --example advanced_plugins_showcase
```

## Use Cases

- Integration with external systems (DB, API, Queue, etc.)
- Custom functions not available in engine
- Domain-specific operations
- Reusable business logic
