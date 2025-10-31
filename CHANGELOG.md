# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.0] - 2025-10-16

### 🔥 Added - Major Plugin System Release

#### Plugin Architecture
- **Modular Plugin System**: Introduced comprehensive plugin architecture with lifecycle management
- **RulePlugin Trait**: Standardized plugin interface with register_actions, register_functions, health_check, and unload methods
- **Plugin Metadata**: Complete plugin information system with name, version, description, author, state, and health tracking
- **Plugin Manager**: Centralized plugin management with configuration and monitoring capabilities

#### Built-in Plugin Suite (44+ Actions, 33+ Functions)
- **StringUtilsPlugin**: 8 actions (ToUpperCase, ToLowerCase, StringTrim, StringLength, StringContains, StringReplace, StringSplit, StringJoin) + 5 functions (concat, repeat, substring, padLeft, padRight)
- **MathUtilsPlugin**: 10 actions (Add, Subtract, Multiply, Divide, Modulo, Power, Abs, Round, Ceil, Floor) + 6 functions (max, min, sqrt, sum, avg, random)
- **DateUtilsPlugin**: 8 actions (CurrentDate, CurrentTime, FormatDate, ParseDate, AddDays, AddHours, DateDiff, IsWeekend) + 7 functions (now, today, dayOfWeek, dayOfYear, year, month, day)
- **ValidationPlugin**: 8 actions (ValidateEmail, ValidatePhone, ValidateUrl, ValidateRegex, ValidateRange, ValidateLength, ValidateNotEmpty, ValidateNumeric) + 6 functions (isEmail, isPhone, isUrl, isNumeric, isEmpty, inRange)
- **CollectionUtilsPlugin**: 10 actions (ArrayLength, ArrayPush, ArrayPop, ArraySort, ArrayFilter, ArrayMap, ArrayFind, ObjectKeys, ObjectValues, ObjectMerge) + 9 functions (length, contains, first, last, reverse, join, slice, keys, values)

#### Plugin Features
- **Health Monitoring**: Real-time plugin health checks with Healthy, Warning, and Error states
- **Dynamic Loading**: Plugin registration and unloading capabilities
- **Error Handling**: Comprehensive error handling for plugin operations
- **Type Safety**: Full Rust type safety for all plugin operations
- **Parameter Validation**: Robust parameter validation and type conversion

#### Examples and Documentation
- **builtin_plugins_demo.rs**: Comprehensive example demonstrating all built-in plugins
- **Plugin System Documentation**: Complete documentation for creating custom plugins
- **Integration Examples**: Real-world examples showing plugin usage in business rules

### 🔧 Enhanced
- **Engine API**: Extended RustRuleEngine with plugin registration methods
- **Function System**: Enhanced function registration with plugin support
- **Action System**: Improved action handler system for plugin integration
- **Error Messages**: Better error messages for plugin-related operations

### 🛠 Technical Improvements
- **Module Organization**: New `src/plugins/` module structure for built-in plugins
- **Export System**: Clean plugin exports through `plugins` module
- **Integration**: Seamless integration between plugins and core rule engine
- **Performance**: Optimized plugin execution with minimal overhead

### 📚 Documentation
- **README Update**: Completely rewritten README focusing on Plugin System v0.9.0
- **API Documentation**: Comprehensive documentation for all plugin interfaces
- **Examples**: Multiple examples showing plugin usage patterns

### 🔄 Breaking Changes
- **Version Bump**: Updated to v0.9.0 to reflect major plugin system addition
- **New Dependencies**: Added `regex` and `chrono` dependencies for built-in plugins

### 🚀 Performance
- **Plugin Efficiency**: Minimal overhead plugin system with lazy loading
- **Memory Usage**: Efficient memory usage with plugin lifecycle management
- **Execution Speed**: Fast plugin action and function execution

### 🧪 Testing
- **Plugin Tests**: Comprehensive test suite for all built-in plugins
- **Integration Tests**: Tests for plugin interaction with rule engine
- **Example Tests**: All examples include test cases

## [0.8.0] - Previous Release
### 🌊 Advanced Workflow Engine
- Comprehensive workflow management with agenda groups and scheduled tasks

## [0.7.1] - Previous Release  
### 🚨 Advanced Action Handlers
- Custom action execution system for external integrations

## [0.7.0] - Previous Release
### 🧩 Advanced Pattern Matching
- EXISTS, NOT, FORALL patterns for complex conditional logic

## [0.6.0] - Previous Release
### 🎯 Rule Attributes
- Advanced rule attributes including agenda groups, activation groups, lock-on-active, and date-based rules

## [0.5.1] - 2025-10-10

### 🔄 Added No-Loop Protection (Major Feature)
- **🔄 No-Loop Attribute Support**: Drools-compatible `no-loop` attribute to prevent infinite rule self-activation
- **📝 GRL Parser Enhancement**: Full support for `no-loop` in GRL syntax
  - `rule "Name" no-loop salience 10 { ... }` ✅
  - `rule "Name" salience 10 no-loop { ... }` ✅  
- **🧠 Engine Logic Enhancement**: Per-cycle rule firing tracking to enforce no-loop behavior
- **🎯 Builder API**: New `.with_no_loop(true)` method for programmatic rule creation
- **🧪 Comprehensive Testing**: Full test suite with parsing and execution validation
- **📚 Documentation**: Complete examples and real-world use cases

### Technical Details
- Added `no_loop: bool` field to `Rule` struct
- Enhanced rule execution engine with `fired_rules_in_cycle` tracking
- Updated GRL regex to parse no-loop attribute in multiple positions
- Backward compatible: existing rules default to `no_loop=false`

### Examples Added
- `examples/no_loop_demo.rs`: Programmatic API demonstration  
- `examples/grl_no_loop_demo.rs`: GRL parsing and execution examples
- `examples/rules/no_loop_test.grl`: Sample rule files with various no-loop patterns

## [0.4.1] - 2025-10-10

### Enhanced
- **🧠 Advanced GRL Parser with Complex Nested Conditions Support**
  - Deep nested parentheses parsing: `(((A && B) || (C && D)))`
  - Proper operator precedence handling: OR (lowest) → AND (higher) → Single conditions (highest)
  - Smart logical operator splitting that respects parentheses nesting
  - Enhanced field name support: both PascalCase (`User.Age`) and lowercase (`user.age`)
  - New function calls support: `set(field, value)` and `add(value)` actions
  - Backward compatibility with existing rule formats

### Added
- **📝 Comprehensive Rule Examples Collection**
  - `examples/rules/` directory with organized rule samples
  - `test_complex_rule.grl`: Complex nested business logic examples
  - `simple_business_rules.grl`: Basic e-commerce rule patterns
  - `advanced_nested_rules.grl`: Premium customer and seasonal campaigns
  - `legacy_format_rules.grl`: Backward compatibility demonstration
  - Comprehensive test suite for all rule complexity levels

### Improved
- **🔧 Parser Architecture Consolidation**
  - Removed duplicate `grl_parser.rs` (331 lines of redundant code)
  - Single source of truth with enhanced `grl.rs` parser
  - All examples and benchmarks migrated to use unified parser
  - Better error handling and more detailed parse error messages
  - Enhanced value type parsing: proper Boolean, Integer, and String handling

### Fixed
- **🐛 Parsing Issues Resolution**
  - Fixed parentheses stripping for conditions like `(user.age >= 18)`
  - Corrected value parsing for complex expressions (no more extra parentheses)
  - Proper nested condition structure building
  - Enhanced regex patterns for field name matching
  - Improved rule name extraction for quoted rule names

### Technical Details
- All 20 unit tests + 5 integration tests + 3 doc tests pass
- Performance validated with complex rule parsing examples
- Memory usage optimized through code consolidation
- Enhanced test coverage for parser edge cases

## [0.4.0] - 2025-01-27

### Added
- **🌐 Distributed & Cloud Architecture**: Complete enterprise-grade distributed rule engine capabilities
  - Multi-node architecture with specialized worker roles (validation, pricing, loyalty)
  - Container orchestration with Docker and Kubernetes deployment manifests
  - Cloud platform integration guides for AWS EKS, Google GKE, and Azure AKS
  - Redis-based shared state management for distributed facts coordination
  - Load balancing with Nginx/HAProxy configurations and health check endpoints
  - Auto-scaling policies and resource management for production deployments
  - Performance monitoring demonstrating 3x speedup (1.4s → 458ms) with parallel processing
- **📚 Comprehensive Documentation in English**:
  - `docs/distributed_features_guide.md`: Technical implementation guide with code examples
  - `docs/distributed_explained.md`: Real-world scenarios and cost analysis
  - `examples/distributed_concept_demo.rs`: Working demonstration with performance comparisons
  - Enhanced README with Table of Contents and organized examples by category
  - Complete Vietnamese to English conversion across all documentation files
- **🚀 Production-Ready Examples**:
  - Cloud deployment templates for major providers
  - Container configurations with resource limits and security policies
  - Cost optimization strategies and scaling recommendations
  - Fault tolerance patterns and disaster recovery guides

### Enhanced
- **README.md**: Added comprehensive distributed section with navigation and organized examples
- **Documentation**: Standardized all content to English for international accessibility
- **Examples Organization**: Categorized demos into Core Features, Performance & Scaling, Advanced Features, and Production Examples

### Performance
- **Distributed Processing**: 3x performance improvement through parallel execution
- **Scalability**: Support for horizontal scaling with cloud-native deployment patterns
- **Fault Tolerance**: Resilient architecture with graceful degradation capabilities

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
