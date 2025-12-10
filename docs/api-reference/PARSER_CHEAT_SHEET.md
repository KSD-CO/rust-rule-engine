# Parser Module System - Cheat Sheet

## ONE-LINE ANSWER

**Parser distinguishes modules using comment markers (`;; MODULE: NAME`) - it searches backward from each rule to find the nearest marker and assigns that rule to the module.**

---

## 10-Second Summary

```
;; MODULE: SENSORS      ← Comment marker tells parser
                         all rules below belong to SENSORS
rule "Rule1" { ... }   ← Assigned to SENSORS
rule "Rule2" { ... }   ← Assigned to SENSORS

;; MODULE: CONTROL     ← New marker
rule "Rule3" { ... }   ← Assigned to CONTROL
```

**How it works**: For each rule, search backward for `;; MODULE:`, extract the module name, assign the rule.

---

## Required Syntax

### Module Definition
```grl
defmodule MODULE_NAME {
  export: all           ;; or: none, or pattern
  import: SOURCE (rules * (templates *))
}
```

### Module Context Marker
```grl
;; MODULE: MODULE_NAME
```

**CRITICAL**: Must use **double semicolon** (`;; `) NOT single (`;`)

---

## Step-by-Step Algorithm

```
For each rule:
  1. Find rule position in file
  2. Look at text BEFORE this rule
  3. Search backward for ";; MODULE: "
  4. If found:
       - Get module name (first word after marker)
       - Assign rule to that module
  5. If NOT found:
       - Assign rule to "MAIN"
```

---

## Visualization

```
File Order:
─────────────────────────────────────────
defmodule SENSORS { ... }          (1) Module definition
                                       (declares exports/imports)
;; MODULE: SENSORS                  (2) Marker
rule "Rule1" { ... }                (3) Rule assigned to SENSORS
rule "Rule2" { ... }                    Rule assigned to SENSORS
                                       
;; MODULE: CONTROL                  (4) New marker
defmodule CONTROL { ... }           (5) Module definition
rule "Rule3" { ... }                (6) Rule assigned to CONTROL
─────────────────────────────────────────

Parser Output:
  Rule1 → SENSORS
  Rule2 → SENSORS
  Rule3 → CONTROL
```

---

## Three Files Created

| File | Lines | Size | Purpose |
|------|-------|------|---------|
| MODULE_PARSING_GUIDE.md | 564 | 14KB | Complete technical reference |
| PARSER_MODULE_QUICK_REF.md | 354 | 11KB | Quick reference & examples |
| MODULE_PARSING_EXAMPLES.md | 522 | 14KB | Real-world examples & diagrams |
| **TOTAL** | **1440** | **39KB** | Complete documentation |

---

## Perfect Template

```grl
;; ============================================
;; MODULE DEFINITIONS
;; ============================================

defmodule SENSORS {
  export: all
}

defmodule CONTROL {
  import: SENSORS (rules * (templates *))
  export: all
}

;; ============================================
;; MODULE: SENSORS
;; ============================================

rule "SensorRule1" { ... }
rule "SensorRule2" { ... }

;; ============================================
;; MODULE: CONTROL
;; ============================================

rule "ControlRule1" { ... }
rule "ControlRule2" { ... }
```

---

## Key Points

| Aspect | Details |
|--------|---------|
| **Marker Format** | `;; MODULE: NAME` (exact, double ;;) |
| **Search Direction** | Always backward from rule |
| **Name Extraction** | First word after "MODULE:" |
| **Default** | "MAIN" if no marker found |
| **Scope** | Marker applies until next marker |
| **Module Names** | Must start with UPPERCASE or _ |
| **Performance** | ~0.75ms for 14 rules |

---

## Common Mistakes

```grl
❌ WRONG: ; MODULE: SENSORS       (single semicolon)
✅ RIGHT: ;; MODULE: SENSORS      (double semicolon)

❌ WRONG: rule "X" { ... }
          ;; MODULE: SENSORS    (marker after rule)
✅ RIGHT: ;; MODULE: SENSORS
          rule "X" { ... }       (marker before rule)

❌ WRONG: ;; MODULE: sensors      (lowercase)
✅ RIGHT: ;; MODULE: SENSORS      (uppercase)

❌ WRONG: defmodule SENSORS { ... }
          rule "X" { ... }        (no marker, rule in MAIN)
✅ RIGHT: defmodule SENSORS { ... }
          ;; MODULE: SENSORS
          rule "X" { ... }        (rule in SENSORS)
```

---

## How to Verify

```rust
use rust_rule_engine::parser::grl::GRLParser;

let grl = "your GRL content";
let parsed = GRLParser::parse_with_modules(grl)?;

// Check rule → module mapping
for (rule_name, module_name) in &parsed.rule_modules {
    println!("{} → {}", rule_name, module_name);
}
```

---

## Parser Flow

```
INPUT: GRL Text
   ↓
STEP 1: Find defmodule blocks
   ↓
STEP 2: Register modules in ModuleManager
   ↓
STEP 3: Remove defmodule blocks from text
   ↓
STEP 4: Parse rules (without module blocks)
   ↓
STEP 5: Search backward from each rule for ;; MODULE: marker
   ↓
STEP 6: Extract module name, assign rule
   ↓
OUTPUT: ParsedGRL {
  rules: Vec<Rule>,
  module_manager: ModuleManager,
  rule_modules: HashMap<rule_name → module_name>
}
```

---

## Example: smart_home.grl

```grl
;; MODULE: SENSORS
rule "CheckHighTemperature" { ... }  → SENSORS
rule "CheckLowTemperature" { ... }   → SENSORS
rule "CheckHighHumidity" { ... }     → SENSORS

;; MODULE: CONTROL
rule "ActivateCooling" { ... }       → CONTROL
rule "ActivateHeating" { ... }       → CONTROL

;; MODULE: ALERT
rule "CriticalTemperature" { ... }   → ALERT
```

---

## Regex Used by Parser

```
Module Definition:  defmodule\s+([A-Z_]\w*)\s*\{([^}]*)\}
Context Marker:     ;; MODULE:
```

---

## Remember

✅ Parser uses **comment markers** to distinguish modules
✅ Searches **backward** from rule for nearest marker
✅ Extracts **first word** after "MODULE:"
✅ Defaults to **"MAIN"** if no marker
✅ **Double semicolon** required: `;;`
✅ Marker must appear **BEFORE** rule

---

## More Info

- Full guide: `MODULE_PARSING_GUIDE.md`
- Quick ref: `PARSER_MODULE_QUICK_REF.md`
- Examples: `MODULE_PARSING_EXAMPLES.md`
- Syntax: `GRL_SYNTAX.md`
- Feature status: `MODULE_SYSTEM_ANALYSIS.md`

