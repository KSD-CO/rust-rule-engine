//! Module đo coverage cho rule engine
//! Lưu lại thông tin rule đã được test, facts đã test, và sinh báo cáo coverage

use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct RuleCoverage {
    /// Tên rule -> số lần được kích hoạt
    pub rule_hits: HashMap<String, usize>,
    /// Tên rule -> facts đã test
    pub rule_facts: HashMap<String, HashSet<String>>,
    /// Facts đã test toàn bộ
    pub tested_facts: HashSet<String>,
}

impl RuleCoverage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ghi nhận rule được kích hoạt với facts
    pub fn record_hit(&mut self, rule_name: &str, facts_id: &str) {
        *self.rule_hits.entry(rule_name.to_string()).or_insert(0) += 1;
        self.rule_facts.entry(rule_name.to_string())
            .or_insert_with(HashSet::new)
            .insert(facts_id.to_string());
        self.tested_facts.insert(facts_id.to_string());
    }

    /// Sinh báo cáo coverage, cảnh báo rule chưa được test
    pub fn report(&self, all_rules: &[String]) -> String {
        let mut report = String::new();
        report.push_str("Rule Coverage Report\n====================\n");
        let mut untested = Vec::new();
        for rule in all_rules {
            let hits = self.rule_hits.get(rule).copied().unwrap_or(0);
            let facts = self.rule_facts.get(rule).map(|f| f.len()).unwrap_or(0);
            report.push_str(&format!(
                "Rule: {} | Hits: {} | Facts tested: {}\n",
                rule, hits, facts
            ));
            if hits == 0 {
                untested.push(rule.clone());
            }
        }
        report.push_str(&format!("Total facts tested: {}\n", self.tested_facts.len()));
        if !untested.is_empty() {
            report.push_str("\n⚠️ Cảnh báo: Các rule chưa được test:\n");
            for rule in untested {
                report.push_str(&format!("  - {}\n", rule));
            }
        }
        report
    }
}

// TODO: Thêm hàm sinh test case cho facts và chạy toàn bộ rule để đo coverage

/// Lấy tất cả điều kiện Single từ ConditionGroup

fn flatten_conditions(group: &crate::engine::rule::ConditionGroup) -> Vec<crate::engine::rule::Condition> {
    use crate::engine::rule::ConditionGroup;
    let mut out = Vec::new();
    match group {
        ConditionGroup::Single(cond) => out.push(cond.clone()),
        ConditionGroup::Compound { left, right, .. } => {
            out.extend(flatten_conditions(left));
            out.extend(flatten_conditions(right));
        }
        ConditionGroup::Not(inner) | ConditionGroup::Exists(inner) | ConditionGroup::Forall(inner) => {
            out.extend(flatten_conditions(inner));
        }
        ConditionGroup::Accumulate { .. } => {
            // Accumulate doesn't have simple single conditions to flatten
            // Skip for now
        }
    }
    out
}

/// Sinh facts mẫu cho một rule dựa trên nhiều kiểu dữ liệu và kết hợp nhiều điều kiện
pub fn generate_test_facts_for_rule(rule: &crate::engine::rule::Rule) -> Vec<crate::Facts> {
    use crate::types::Value;
    let conds = flatten_conditions(&rule.conditions);
    let mut test_facts = Vec::new();

    // Sinh facts cho từng điều kiện riêng lẻ
    for cond in &conds {
        let mut facts = crate::Facts::new();
        let field = cond.field.clone();
        match &cond.value {
            Value::Integer(i) => {
                facts.set(&field, Value::Integer(*i));
                test_facts.push(facts.clone());
                facts.set(&field, Value::Integer(i + 1));
                test_facts.push(facts.clone());
            }
            Value::Boolean(b) => {
                facts.set(&field, Value::Boolean(*b));
                test_facts.push(facts.clone());
                facts.set(&field, Value::Boolean(!b));
                test_facts.push(facts.clone());
            }
            Value::String(s) => {
                facts.set(&field, Value::String(s.clone()));
                test_facts.push(facts.clone());
                facts.set(&field, Value::String("other_value".to_string()));
                test_facts.push(facts.clone());
            }
            _ => {}
        }
    }

    // Sinh facts kết hợp nhiều điều kiện (all true)
    if !conds.is_empty() {
        let mut facts = crate::Facts::new();
        for cond in &conds {
            let field = cond.field.clone();
            match &cond.value {
                Value::Integer(i) => facts.set(&field, Value::Integer(*i)),
                Value::Boolean(b) => facts.set(&field, Value::Boolean(*b)),
                Value::String(s) => facts.set(&field, Value::String(s.clone())),
                _ => {}
            }
        }
        test_facts.push(facts);
    }

    test_facts
}
