/// Demo: Expression Evaluation - CLIPS-style
///
/// This example demonstrates runtime expression evaluation in GRL rules:
/// - Arithmetic operations: +, -, *, /, %
/// - Field references in expressions (Order.quantity * Order.price)
/// - CLIPS-like (bind ?total (* ?quantity ?price))

use rust_rule_engine::engine::engine::RustRuleEngine;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};
use rust_rule_engine::{Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🧮 Expression Evaluation Demo - CLIPS-style");
    println!("============================================\n");

    // Load and parse GRL rules
    let grl_content = std::fs::read_to_string("examples/rules/expression_demo.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let mut kb = KnowledgeBase::new("expression_demo");
    for rule in &rules {
        kb.add_rule(rule.clone());
    }

    let mut engine = RustRuleEngine::new(kb);

    // Example 1: Calculate order total
    println!("📋 Example 1: Calculate Order Total");
    println!("------------------------------------");

    let mut facts = Facts::new();
    facts.set("Order.quantity", Value::Integer(10));
    facts.set("Order.price", Value::Integer(100));

    println!("Before execution:");
    println!("  Order.quantity: {:?}", facts.get("Order.quantity"));
    println!("  Order.price: {:?}", facts.get("Order.price"));

    engine.execute(&mut facts)?;

    println!("\nAfter execution:");
    println!("  Order.total (10 * 100): {:?}", facts.get("Order.total"));
    println!("  Order.discount (1000 * 0.1): {:?}", facts.get("Order.discount"));
    println!("  Order.final (1000 - 100): {:?}", facts.get("Order.final"));
    println!("  Order.tax (900 * 0.08): {:?}", facts.get("Order.tax"));
    println!("  Order.grandTotal (900 + 72): {:?}", facts.get("Order.grandTotal"));
    println!("  Order.bulkSavings (1000 * 0.15): {:?}", facts.get("Order.bulkSavings"));

    // Example 2: Different quantities
    println!("\n📋 Example 2: Small Order (No Bulk Discount)");
    println!("---------------------------------------------");

    let mut kb2 = KnowledgeBase::new("expression_demo");
    for rule in &rules {
        kb2.add_rule(rule.clone());
    }
    let mut engine2 = RustRuleEngine::new(kb2);

    let mut facts2 = Facts::new();
    facts2.set("Order.quantity", Value::Integer(5));
    facts2.set("Order.price", Value::Integer(50));

    println!("Before execution:");
    println!("  Order.quantity: {:?}", facts2.get("Order.quantity"));
    println!("  Order.price: {:?}", facts2.get("Order.price"));

    engine2.execute(&mut facts2)?;

    println!("\nAfter execution:");
    println!("  Order.total (5 * 50): {:?}", facts2.get("Order.total"));
    println!("  Order.discount (250 * 0.1): {:?}", facts2.get("Order.discount"));
    println!("  Order.final (250 - 25): {:?}", facts2.get("Order.final"));
    println!("  Order.tax (225 * 0.08): {:?}", facts2.get("Order.tax"));
    println!("  Order.grandTotal (225 + 18): {:?}", facts2.get("Order.grandTotal"));
    println!("  Order.bulkSavings: {:?}", facts2.get("Order.bulkSavings")); // Should be None

    // Example 3: RETE Engine Test
    println!("\n📋 Example 3: RETE Engine Test");
    println!("--------------------------------");

    let mut rete = IncrementalEngine::new();
    let count = GrlReteLoader::load_from_string(&grl_content, &mut rete)?;
    println!("Loaded {} rules into RETE\n", count);

    let mut order = TypedFacts::new();
    order.set("quantity", 15i64);
    order.set("price", 80i64);

    println!("Before (RETE):");
    println!("  quantity: {:?}, price: {:?}", order.get("quantity"), order.get("price"));

    let handle = rete.insert("Order".to_string(), order);
    let fired = rete.fire_all();
    println!("\nFired {} rules", fired.len());

    if let Some(fact) = rete.working_memory().get(&handle) {
        println!("\nAfter (RETE):");
        println!("  total: {:?}", fact.data.get("total"));
        println!("  discount: {:?}", fact.data.get("discount"));
        println!("  final: {:?}", fact.data.get("final"));
        println!("  tax: {:?}", fact.data.get("tax"));
        println!("  grandTotal: {:?}", fact.data.get("grandTotal"));
    }

    // Summary
    println!("\n✨ Expression Evaluation Summary");
    println!("===============================");
    println!("✅ Arithmetic operators: +, -, *, /, %");
    println!("✅ Field references: Order.quantity, Order.price");
    println!("✅ Runtime evaluation: Expressions evaluated when rule fires");
    println!("✅ Type preservation: Integer * Integer = Integer");
    println!("✅ Float operations: Integer * 0.1 = Float");
    println!("✅ Chained expressions: Order.total * 0.1 works perfectly");
    println!("✅ Works with both RustRuleEngine and RETE!");
    println!("\n📖 Similar to Drools DRL:");
    println!("   Drools: $o.total = $o.quantity * $o.price");
    println!("   Rust Rule Engine: Order.total = Order.quantity * Order.price");

    Ok(())
}
