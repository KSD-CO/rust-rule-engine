//! Example: Sử dụng RuleCoverage để đo coverage khi chạy rule

use rust_rule_engine::engine::coverage::RuleCoverage;

fn main() {
    let mut coverage = RuleCoverage::new();

    // Giả lập chạy rule với facts
    coverage.record_hit("rule_discount", "facts_1");
    coverage.record_hit("rule_discount", "facts_2");
    coverage.record_hit("rule_vip", "facts_1");
    coverage.record_hit("rule_vip", "facts_3");
    coverage.record_hit("rule_discount", "facts_3");

    // Danh sách tất cả rules trong knowledge base
    let all_rules = vec![
        "rule_discount".to_string(),
        "rule_vip".to_string(),
        "rule_free_shipping".to_string(),  // Rule chưa được test
    ];

    // In báo cáo coverage ra console
    println!("{}", coverage.report(&all_rules));
}
