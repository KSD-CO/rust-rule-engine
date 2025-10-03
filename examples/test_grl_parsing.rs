use rust_rule_engine::parser::grl::GRLParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing GRL Parser Edge Cases");
    println!("=================================");

    // Test các GRL patterns khác nhau
    let test_cases = vec![
        (
            "Simple condition",
            r#"rule "Test1" {
when
User.Age > 18
then
User.setStatus("adult");
}"#,
        ),
        (
            "Compound condition với &&",
            r#"rule "Test2" {
when
User.Country == "US" && User.SpendingTotal >= 1000
then
User.setIsVIP(true);
}"#,
        ),
        (
            "Assignment action",
            r#"rule "Test3" {
when
User.Age >= 18
then
User.IsAdult = true;
}"#,
        ),
        (
            "Method call với string param",
            r#"rule "Test4" {
when
User.Country == "US"
then
User.setVIPLevel("Gold");
}"#,
        ),
        (
            "Số without quotes",
            r#"rule "Test5" {
when
User.Age >= 18
then
User.setAge(25);
}"#,
        ),
        (
            "Boolean value",
            r#"rule "Test6" {
when
User.IsActive == true
then
User.setStatus("active");
}"#,
        ),
    ];

    for (desc, grl) in test_cases.iter() {
        println!("\n🧪 Testing: {}", desc);
        println!("GRL:");
        println!("{}", grl);

        match GRLParser::parse_rules(grl) {
            Ok(rules) => {
                println!("✅ Parsed successfully: {} rules", rules.len());
                if !rules.is_empty() {
                    println!("   Rule name: {}", rules[0].name);
                    println!("   Actions: {} actions", rules[0].actions.len());
                }
            }
            Err(e) => {
                println!("❌ Parse failed: {}", e);
            }
        }
        println!("{}", "─".repeat(50));
    }

    Ok(())
}
