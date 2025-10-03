# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-10-02

### Added
- **Rule Templates System**: Complete template-based rule generation with parameterization
  - `RuleTemplate` with parameter definitions and substitution
  - `TemplateManager` for template management and JSON serialization
  - `ParameterType` enum supporting String, Number, Boolean, Array types
  - Template instantiation with parameter validation
  - Bulk rule generation from templates
- New examples: `rule_templates_concept.rs`, `rule_templates_demo.rs`
- Enhanced GRL parsing test cases

### Fixed
- **Critical Parser Fix**: GRL parser now supports quoted rule names
  - Updated regex from `r"rule\s+\w+[^}]*\}"` to `r#"rule\s+(?:"[^"]+"|[a-zA-Z_]\w*)[^}]*\}"`
  - Both quoted (`"RuleName"`) and unquoted (`RuleName`) rule names now supported
  - All existing rule parsing continues to work without regression

### Improved
- Enhanced template system documentation
- Added comprehensive test coverage for rule templates
- Better error handling in template parameter validation

## [0.1.2] - 2025-10-01

### Fixed
- **Critical Fix**: Resolved infinite loop issue in rule execution
- Fixed custom functions to properly update facts in memory using `facts.set_nested()`
- Added guard conditions to example rules to prevent re-firing
- Improved engine configuration with `max_cycles` setting to prevent infinite loops

### Added
- Enhanced inline rules demo with proper fact updates
- Added `Customer.setLoyaltyBonusApplied()` and `Transaction.setRiskProcessed()` functions
- Better debugging output for rule execution cycles

### Changed
- Default `max_cycles` in examples set to 1 to prevent infinite loops
- Custom functions now properly modify facts state instead of just returning strings

## [0.1.1] - 2025-10-01

### Added
- Published to crates.io with complete functionality
- External package validation examples

## [0.1.0] - 2025-10-01

### Added
- **GRL (Grule Rule Language) Support**: Full parser for Grule-compatible syntax
- **Method Calls**: Support for `$Object.method(args)` style method invocations
- **Property Access**: Object property access with `$Object.Property` syntax
- **Arithmetic Expressions**: Complex calculations in conditions and actions
- **Knowledge Base**: Centralized rule management with salience-based execution
- **Facts System**: Working memory for complex object interactions
- **Grule Engine**: High-performance rule execution engine
- **Engine Configuration**: Configurable timeouts, max cycles, and debug modes
- **Execution Statistics**: Detailed performance metrics and debugging info
- **Helper Functions**: `FactHelper` for common object creation patterns
- **Type Safety**: Comprehensive type system with Value enum
- **Error Handling**: Detailed error types and handling
- **Examples**: Real-world scenarios including e-commerce and fraud detection
- **Documentation**: Comprehensive README and inline documentation

### Features
- Rule parsing from GRL syntax
- Condition evaluation with complex operators (`==`, `!=`, `>`, `<`, `>=`, `<=`)
- Compound conditions with logical operators (`&&`, `||`)
- Action execution with field assignments
- Method calls on objects with argument evaluation
- Function calls like `update()` and `Log()`
- Salience-based rule prioritization
- Cycle detection and prevention
- Debug mode with detailed execution logging
- Performance monitoring and statistics

### Examples
- Basic method calls demo
- Advanced SpeedUp rule with object interactions
- E-commerce discount rules
- Fraud detection scenarios
- Condition debugging utilities

### Core Components
- `KnowledgeBase`: Rule storage and management
- `Facts`: Working memory and data objects
- `RustRuleEngine`: Rule execution engine
- `GRLParser`: Grule Rule Language parser
- `FactHelper`: Utility functions for object creation
- `Value`: Flexible data type system
- `ActionType`: Action execution system

### Architecture
- Modular design with clear separation of concerns
- High-performance execution with minimal overhead
- Memory-efficient data structures
- Thread-safe design for concurrent usage
- Extensible action and function system

## [Unreleased]

### Planned
- Enhanced GRL parser with full Grule compatibility
- RETE algorithm implementation for advanced pattern matching
- Rule debugging and step-through capabilities
- Web dashboard for rule management
- Hot reload functionality for dynamic rule updates
- Distributed rule execution across multiple nodes
- Visual rule editor with drag-and-drop interface
- Additional integrations (databases, message queues)
