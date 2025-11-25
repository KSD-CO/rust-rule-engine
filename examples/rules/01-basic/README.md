# Basic GRL Rules

Basic rule files to learn GRL syntax and concepts.

## Files

### 1. simple_business_rules.grl
**3 rules** - Simplest examples

Rules:
- Check adult (age >= 18)
- VIP discount (VIP status)
- Senior discount (age >= 65)

**Learn:**
- Basic rule structure
- Simple conditions
- Variable assignment

**Example:**
```grl
rule "CheckAdult" "Check if person is adult" {
    when
        Person.Age >= 18
    then
        Person.IsAdult = true;
}
```

### 2. simple_patterns.grl
**3 rules** - Basic pattern matching

Rules:
- Test exists pattern
- Test forall pattern
- Test !exists (not exists) pattern

**Learn:**
- Conditional elements
- Pattern matching basics
- Boolean logic

### 3. grule_demo.grl
**5 rules** - Overall demo

Rules:
- Adult check
- VIP check
- Senior citizen
- Category assignment
- Discount calculation

**Learn:**
- Multiple rules interaction
- Rule chaining
- Real business logic

### 4. expression_demo.grl
**3 rules** - Arithmetic expressions

Rules:
- Calculate order total
- Calculate tax
- Apply discount

**Learn:**
- Arithmetic operations (+, -, *, /)
- Mathematical expressions
- Computed values

**Example:**
```grl
rule "CalculateTotal" {
    when
        Order.Quantity > 0 && Order.Price > 0
    then
        Order.Total = Order.Quantity * Order.Price;
}
```

### 5. ecommerce.grl
**6 rules** - E-commerce business rules

Rules:
- Volume discount (>100 items)
- Loyalty discount
- Weekend special
- Seasonal sale
- First-time buyer bonus
- Free shipping threshold

**Learn:**
- Real business scenarios
- Multiple discount rules
- Complex conditions

### 6. method_calls.grl
**4 rules** - Calling Rust methods

Rules:
- Call simple method
- Call method with parameters
- Call getter/setter
- Chain method calls

**Learn:**
- Method invocation syntax
- Passing arguments
- Return values

**Example:**
```grl
rule "CallMethod" {
    when
        User.IsActive() == true
    then
        User.UpdateLastLogin();
        Log("User logged in");
}
```

## Learning Order

### Step 1: Basics
1. `simple_business_rules.grl` - Understand rule structure
2. `expression_demo.grl` - Learn arithmetic

### Step 2: Patterns
3. `simple_patterns.grl` - Pattern matching
4. `method_calls.grl` - Method calls

### Step 3: Real-world
5. `grule_demo.grl` - Multiple rules
6. `ecommerce.grl` - Business use case

## Run with Rust Examples

```bash
# Grule demo
cargo run --example grule_demo

# Expression demo
cargo run --example expression_demo

# Method calls
cargo run --example method_calls_demo
```

## GRL Basics Cheatsheet

### Rule Structure
```grl
rule "Name" "Description" salience 10 {
    when
        Condition1 && Condition2
    then
        Action1;
        Action2;
}
```

### Operators
- Comparison: `==`, `!=`, `>`, `<`, `>=`, `<=`
- Boolean: `&&`, `||`, `!`
- Arithmetic: `+`, `-`, `*`, `/`, `%`

### Common Patterns
```grl
// Simple condition
when
    Person.Age >= 18

// Multiple conditions
when
    Person.Age >= 18 && Person.VIP == true

// Negation
when
    !Person.IsBlocked

// Method call in condition
when
    Person.GetAge() >= 18
```

## Next Steps

After mastering basic rules, continue with:
- `02-rete/` - RETE-optimized patterns
- `03-advanced/` - Advanced features
- `04-use-cases/` - Production examples
