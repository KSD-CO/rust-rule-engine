# Module Parsing Guide - How the Parser Reads & Distinguishes Modules

## 📖 Overview

The GRL parser uses a **3-step process** to read and organize rules into modules:

1. **Parse Module Definitions** - Find all `defmodule` blocks
2. **Register Modules** - Create ModuleManager with exports/imports
3. **Extract Module Context** - Assign each rule to its module

---

## Step 1: Parse Module Definitions

### What the Parser Looks For

```grl
defmodule MODULE_NAME {
  export: all
  import: SOURCE_MODULE (rules * (templates *))
}
```

### Regex Pattern Used

```rust
static DEFMODULE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"defmodule\s+([A-Z_]\w*)\s*\{([^}]*)\}"#)
});
```

**This regex captures**:
- `defmodule` keyword
- `MODULE_NAME` (Group 1) - Must start with uppercase or underscore
- `{...}` content (Group 2) - Everything inside braces

### Example Parsing

```grl
defmodule SENSORS {
  export: all
}

defmodule CONTROL {
  import: SENSORS (rules * (templates temperature))
  export: all
}
```

**Parser Output**:
```
Match 1:
  - Group 1: "SENSORS"
  - Group 2: "export: all"

Match 2:
  - Group 1: "CONTROL"
  - Group 2: "import: SENSORS (rules * (templates temperature))\n  export: all"
```

---

## Step 2: Register Modules in ModuleManager

### Process

```rust
fn parse_and_register_module(&self, module_def: &str, manager: &mut ModuleManager) -> Result<()> {
    // Extract module name
    if let Some(captures) = DEFMODULE_REGEX.captures(module_def) {
        let module_name = captures.get(1).unwrap().as_str();  // e.g., "SENSORS"
        let module_body = captures.get(2).unwrap().as_str();  // e.g., "export: all"

        // 1. Create module in manager
        manager.create_module(&module_name)?;
        
        // 2. Parse export directive
        if let Some(export_type) = self.extract_directive(module_body, "export:") {
            let exports = if export_type.trim() == "all" {
                ExportList::All
            } else if export_type.trim() == "none" {
                ExportList::None
            }
            module.set_exports(exports);
        }
        
        // 3. Parse import directives
        for import_line in module_body.lines() {
            if import_line.trim().starts_with("import:") {
                self.parse_import_spec(&module_name, &import_spec, manager)?;
            }
        }
    }
}
```

### What Gets Registered

For this GRL:
```grl
defmodule SENSORS {
  export: all
}

defmodule CONTROL {
  import: SENSORS (rules * (templates temperature))
  export: all
}
```

**ModuleManager State After Registration**:
```
SENSORS
  ├── export: all
  └── imports: []

CONTROL
  ├── export: all
  └── imports: [
        {
          from_module: "SENSORS",
          rules: "*",
          templates: "temperature"
        }
      ]
```

---

## Step 3: Extract Module Context from Comments

### Key Algorithm: `extract_module_from_context()`

This is the **magic** - it assigns rules to modules based on comment markers:

```rust
fn extract_module_from_context(&self, grl_text: &str, rule_name: &str) -> String {
    // Step 1: Find the rule in the file
    if let Some(rule_pos) = grl_text.find(&format!("rule \"{}\"", rule_name)) {
        
        // Step 2: Look backward from rule position to find ;; MODULE: comment
        let before = &grl_text[..rule_pos];
        if let Some(module_pos) = before.rfind(";; MODULE:") {
            
            // Step 3: Extract module name from the comment line
            let after_module_marker = &before[module_pos + 10..];
            if let Some(end_of_line) = after_module_marker.find('\n') {
                let module_line = &after_module_marker[..end_of_line].trim();
                
                // Extract first word (e.g., "SENSORS" from "SENSORS - Temperature Monitoring")
                if let Some(first_word) = module_line.split_whitespace().next() {
                    return first_word.to_string();
                }
            }
        }
    }
    
    // If no comment found, default to MAIN
    "MAIN".to_string()
}
```

### How It Works - Step by Step

```grl
defmodule SENSORS {
  export: all
}

;; ============================================
;; MODULE: SENSORS
;; ============================================

rule "CheckTemperature" salience 100 {
  when temperature.value > 28
  then println("High temp");
}

;; ============================================
;; MODULE: CONTROL
;; ============================================

rule "ActivateCooling" salience 80 {
  when temperature.value > 28
  then hvac.state = "ON";
}
```

### Parsing Process for Each Rule

**For Rule "CheckTemperature":**

1. **Find rule position** in text
   ```
   Position = where 'rule "CheckTemperature"' appears
   ```

2. **Look backward** from that position to find `;; MODULE:`
   ```
   Text before rule:
   "defmodule SENSORS {...}
   
   ;; ============================================
   ;; MODULE: SENSORS        ← FOUND!
   ;; ============================================
   "
   ```

3. **Extract module name** from the comment
   ```
   Line: "MODULE: SENSORS"
   After "MODULE: ": "SENSORS"
   First word: "SENSORS" ← ASSIGNED MODULE
   ```

4. **Result**: `rule_modules.insert("CheckTemperature", "SENSORS")`

---

**For Rule "ActivateCooling":**

1. **Find rule position**
2. **Look backward** from that position to find `;; MODULE:`
   ```
   Text before rule:
   "...
   ;; ============================================
   ;; MODULE: CONTROL        ← FOUND THIS TIME!
   ;; ============================================
   "
   ```

3. **Extract**: "CONTROL"
4. **Result**: `rule_modules.insert("ActivateCooling", "CONTROL")`

---

## Full Parsing Workflow

```rust
pub fn parse_with_modules(grl_text: &str) -> Result<ParsedGRL> {
    // ========== STEP 1: Parse Module Definitions ==========
    // Find all: defmodule NAME { ... }
    for module_match in DEFMODULE_SPLIT_REGEX.find_iter(grl_text) {
        parse_and_register_module(module_def, &mut manager);  // Register in ModuleManager
    }
    
    // ========== STEP 2: Clean Text ==========
    // Remove all defmodule blocks to avoid interference with rule parsing
    let rules_text = DEFMODULE_SPLIT_REGEX.replace_all(grl_text, "");
    
    // ========== STEP 3: Parse Rules ==========
    let rules = parse_multiple_rules(&rules_text);
    
    // ========== STEP 4: Extract Module Context ==========
    // For each rule, find which module it belongs to
    for rule in rules {
        let module_name = extract_module_from_context(grl_text, &rule.name);
        
        // Track: rule_name → module_name
        result.rule_modules.insert(rule.name.clone(), module_name.clone());
        
        // Add rule to module
        manager.get_module_mut(&module_name)?.add_rule(&rule.name);
        
        result.rules.push(rule);
    }
    
    return ParsedGRL {
        rules: Vec<Rule>,           // All parsed rules
        module_manager: ModuleManager,  // Modules with imports/exports
        rule_modules: HashMap<String, String>,  // rule_name → module_name
    };
}
```

---

## Real-World Example

### Input GRL File

```grl
;; ============================================
;; MODULE: SENSORS
;; ============================================

defmodule SENSORS {
  export: all
}

rule "SensorCheckTemperature" salience 100 {
  when temperature.value > 28
  then println("High temp");
}

rule "SensorCheckHumidity" salience 90 {
  when humidity.value > 70
  then println("High humidity");
}

;; ============================================
;; MODULE: CONTROL
;; ============================================

defmodule CONTROL {
  import: SENSORS (rules * (templates temperature))
  export: all
}

rule "ControlActivateCooling" salience 80 {
  when temperature.value > 28 && hvac.state == "OFF"
  then hvac.state = "ON";
}
```

### Parsing Steps

**STEP 1 - Parse Module Definitions:**
```
Found: defmodule SENSORS { export: all }
Found: defmodule CONTROL { import: SENSORS ..., export: all }
```

**STEP 2 - Register Modules:**
```
ModuleManager created with:
  - SENSORS: export all, imports: []
  - CONTROL: export all, imports: [SENSORS]
```

**STEP 3 - Parse Rules (after removing defmodule blocks):**
```
Found: rule "SensorCheckTemperature" { ... }
Found: rule "SensorCheckHumidity" { ... }
Found: rule "ControlActivateCooling" { ... }
```

**STEP 4 - Extract Module Context:**

For `SensorCheckTemperature`:
```
Search backward from rule position...
Found: ";; MODULE: SENSORS"
Extract: "SENSORS"
Assign: SensorCheckTemperature → SENSORS
```

For `SensorCheckHumidity`:
```
Search backward from rule position...
Found: ";; MODULE: SENSORS"  (closest one going backward)
Extract: "SENSORS"
Assign: SensorCheckHumidity → SENSORS
```

For `ControlActivateCooling`:
```
Search backward from rule position...
Found: ";; MODULE: CONTROL"
Extract: "CONTROL"
Assign: ControlActivateCooling → CONTROL
```

### Final Output

```rust
ParsedGRL {
    rules: [
        Rule { name: "SensorCheckTemperature", ... },
        Rule { name: "SensorCheckHumidity", ... },
        Rule { name: "ControlActivateCooling", ... },
    ],
    
    module_manager: ModuleManager {
        modules: {
            "SENSORS": Module { 
                rules: ["SensorCheckTemperature", "SensorCheckHumidity"],
                exports: All,
                imports: []
            },
            "CONTROL": Module {
                rules: ["ControlActivateCooling"],
                exports: All,
                imports: [SENSORS]
            },
            "MAIN": Module {  // Auto-created
                rules: [],
                exports: All,
                imports: []
            }
        }
    },
    
    rule_modules: {
        "SensorCheckTemperature" → "SENSORS",
        "SensorCheckHumidity" → "SENSORS",
        "ControlActivateCooling" → "CONTROL",
    }
}
```

---

## Key Details

### Comment Format Requirements

The parser looks for **exactly**: `;; MODULE: NAME`

```grl
;; ============================================
;; MODULE: SENSORS          ← Parser extracts "SENSORS" (first word after "MODULE:")
;; ============================================
```

**Valid formats** (parser extracts first word):
- `;; MODULE: SENSORS` → "SENSORS"
- `;; MODULE: SENSORS - Temperature monitoring` → "SENSORS"
- `;; MODULE: CONTROL (Decision Making)` → "CONTROL"

**Invalid formats** (won't work):
- `; MODULE: SENSORS` (single semicolon)
- `MODULE: SENSORS` (no semicolons)
- `// MODULE: SENSORS` (C-style comment)
- `MODULE SENSORS` (missing colon)

### Module Name Requirements

From regex: `([A-Z_]\w*)`

- ✅ Must start with **uppercase letter** or underscore
- ✅ Can contain letters, numbers, underscores
- ❌ Cannot start with lowercase
- ❌ Cannot start with numbers

Valid:
- `SENSORS`
- `_PRIVATE`
- `CONTROL_V2`
- `A`

Invalid:
- `sensors` (lowercase)
- `1SENSORS` (starts with number)
- `control-v2` (hyphens not allowed)

### Default Module

If a rule has **no `;; MODULE:` comment** above it, it's assigned to **MAIN**:

```grl
defmodule SENSORS { export: all }

rule "CheckTemperature" { ... }  ;; → SENSORS (has comment above)

rule "RandomRule" { ... }        ;; → MAIN (no comment, not in any module section)
```

---

## Summary Table

| Step | Operation | Input | Output |
|------|-----------|-------|--------|
| 1 | Find defmodule blocks | GRL text | Module definitions |
| 2 | Register modules | Module definitions | ModuleManager |
| 3 | Parse rules | Cleaned GRL (no defmodule) | Rule objects |
| 4 | Extract context | Original GRL + rules | rule_modules HashMap |

---

## Debugging - How to Verify

```rust
use rust_rule_engine::parser::grl::GRLParser;
use std::fs;

let grl = fs::read_to_string("smart_home.grl")?;
let parsed = GRLParser::parse_with_modules(&grl)?;

// Check which module each rule is in
for (rule_name, module_name) in &parsed.rule_modules {
    println!("Rule '{}' belongs to module '{}'", rule_name, module_name);
}

// Check module structure
for module_name in parsed.module_manager.list_modules() {
    let module = parsed.module_manager.get_module(&module_name)?;
    println!("Module: {}", module_name);
    println!("  Rules: {:?}", module.get_rules());
    println!("  Imports: {:?}", module.get_imports());
}
```

---

## Common Issues & Solutions

### Issue 1: Rule Not Assigned to Module

**Problem**: Rule shows as "MAIN" instead of expected module

**Cause**: Missing or wrong comment format

**Solution**: Add correct comment
```grl
;; ============================================
;; MODULE: CONTROL              ← Correct format
;; ============================================

rule "MyRule" { ... }
```

### Issue 2: Module Name Extracted Incorrectly

**Problem**: Module shows as different name than expected

**Cause**: Parser takes first word after "MODULE:"

**Solution**: Make sure first word after "MODULE:" is the actual module name
```grl
;; Good
;; MODULE: SENSORS - Temperature

;; Bad  
;; MODULE: MY_SENSOR_DATA - This extracts "MY_SENSOR_DATA" only
```

### Issue 3: Rules Not in ModuleManager

**Problem**: Rule parsed but not added to module

**Cause**: Rule comments in wrong location

**Solution**: Ensure module section comment appears BEFORE the rules

```grl
;; ============================================
;; MODULE: SENSORS
;; ============================================
rule "Rule1" { ... }  ← Gets SENSORS
rule "Rule2" { ... }  ← Gets SENSORS

;; ============================================
;; MODULE: CONTROL
;; ============================================
rule "Rule3" { ... }  ← Gets CONTROL
```

---

## Performance Notes

- **Regex compilation**: Regexes are pre-compiled with `Lazy` (compiled once at startup)
- **Module parsing**: ~O(n) where n = number of module definitions
- **Context extraction**: ~O(n*m) where n = rules, m = average search distance backward
- **For typical files**: < 1ms parsing time

---

**Key Takeaway**: 
The parser uses **comment markers** (`; MODULE: NAME`) to determine which rules belong to which modules. The `;; MODULE: XXX` comment is a **context marker** that tells the parser "all rules after this belong to the XXX module until the next module marker."

