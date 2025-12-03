.PHONY: help all examples backward-chaining getting-started rete-engine advanced-features plugins performance use-cases advanced-rete misc module-system

# Default target
help:
	@echo "Available targets:"
	@echo "  make all                    - Run all examples"
	@echo "  make getting-started        - Run all getting-started examples"
	@echo "  make rete-engine           - Run all RETE engine examples"
	@echo "  make advanced-features     - Run all advanced features examples"
	@echo "  make plugins               - Run all plugin examples"
	@echo "  make performance           - Run all performance examples"
	@echo "  make use-cases             - Run all use-case examples"
	@echo "  make advanced-rete         - Run all advanced RETE examples"
	@echo "  make misc                  - Run all misc examples"
	@echo "  make backward-chaining     - Run all backward-chaining examples"
	@echo "  make module-system         - Run all module-system examples"
	@echo ""
	@echo "Individual examples:"
	@echo "  make fraud_detection"
	@echo "  make grule_demo"
	@echo "  make simple_query_demo"
	@echo "  ... (and many more, see Cargo.toml for full list)"

# Run all examples
all: getting-started rete-engine advanced-features plugins performance use-cases advanced-rete misc backward-chaining module-system

# Getting Started Examples (01-getting-started)
getting-started:
	@echo "=== Running Getting Started Examples ==="
	cargo run --example fraud_detection
	cargo run --example grule_demo
	cargo run --example advanced_method_calls
	cargo run --example expression_demo
	cargo run --example generic_method_calls
	cargo run --example inline_rules_demo
	cargo run --example method_calls_demo
	cargo run --example simple_pattern_matching_grl

# RETE Engine Examples (02-rete-engine)
rete-engine:
	@echo "=== Running RETE Engine Examples ==="
	cargo run --example multifield_demo
	cargo run --example rete_call_function_demo
	cargo run --example rete_deffacts_demo
	cargo run --example rete_demo
	cargo run --example rete_grl_demo
	cargo run --example rete_memoization_demo
	cargo run --example rete_multifield_demo
	cargo run --example rete_parse_demo
	cargo run --example rete_template_globals_demo
	cargo run --example rete_typed_facts_demo
	cargo run --example tms_demo

# Advanced Features Examples (03-advanced-features)
advanced-features:
	@echo "=== Running Advanced Features Examples ==="
	cargo run --example accumulate_demo
	cargo run --example accumulate_grl_demo
	cargo run --example action_handlers_demo
	cargo run --example action_handlers_grl_demo
	cargo run --example conflict_resolution_demo
	cargo run --example custom_functions_demo
	cargo run --example grl_no_loop_demo
	cargo run --example multifield_operations_demo
	cargo run --example no_loop_demo
	cargo run --example pattern_matching_from_grl
	cargo run --example retract_demo_rete
	cargo run --example retract_demo
	cargo run --example rule_attributes_demo
	cargo run --example rule_templates_demo

# Plugin Examples (04-plugins)
plugins:
	@echo "=== Running Plugin Examples ==="
	cargo run --example advanced_plugins_showcase
	cargo run --example builtin_plugins_demo
	cargo run --example plugin_system_demo

# Performance Examples (05-performance)
performance:
	@echo "=== Running Performance Examples ==="
	cargo run --example complete_speedup_demo
	cargo run --example distributed_demo
	cargo run --example distributed_vs_single_demo
	cargo run --example financial_stress_test
	cargo run --example parallel_advanced_features_test
	cargo run --example parallel_conditions_test
	cargo run --example parallel_engine_demo
	cargo run --example parallel_performance_demo
	cargo run --example purchasing_rules_parse_benchmark
	cargo run --example purchasing_rules_performance
	cargo run --example quick_engine_comparison

# Use Cases Examples (06-use-cases)
use-cases:
	@echo "=== Running Use Cases Examples ==="
	cargo run --example advanced_workflow_demo
	cargo run --example analytics_demo
	cargo run --example workflow_engine_demo

# Advanced RETE Examples (07-advanced-rete)
advanced-rete:
	@echo "=== Running Advanced RETE Examples ==="
	cargo run --example accumulate_rete_integration
	cargo run --example rete_engine_cached
	cargo run --example rete_p2_advanced_agenda
	cargo run --example rete_p2_working_memory
	cargo run --example rete_p3_incremental
	cargo run --example rete_p3_variable_binding
	cargo run --example rete_ul_drools_style

# Misc Examples (08-misc)
misc:
	@echo "=== Running Misc Examples ==="
	cargo run --example rule_dependency_analysis
	cargo run --example rule_file_functions_demo

# Backward Chaining Examples (09-backward-chaining)
backward-chaining:
	@echo "=== Running Backward Chaining Examples ==="
	cargo run --features backward-chaining --example simple_query_demo
	cargo run --features backward-chaining --example medical_diagnosis_demo
	cargo run --features backward-chaining --example detective_system_demo
	cargo run --features backward-chaining --example multiple_solutions_demo
	cargo run --features backward-chaining --example grl_query_demo
	cargo run --features backward-chaining --example unification_demo
	cargo run --features backward-chaining --example ecommerce_approval_demo
	cargo run --features backward-chaining --example purchasing_flow_demo
	cargo run --features backward-chaining --example loan_approval_demo
	cargo run --features backward-chaining --example family_relations_demo
	cargo run --features backward-chaining --example product_recommendation_demo
	cargo run --features backward-chaining --example rete_index_demo
	cargo run --features backward-chaining --example comprehensive_backward_test
	cargo run --features backward-chaining --example backward_edge_cases_test
	cargo run --features backward-chaining --example backward_critical_missing_tests
	cargo run --features backward-chaining --example access_control_demo
	cargo run --features backward-chaining --example aggregation_demo
	cargo run --features backward-chaining --example grl_aggregation_demo

# Module System Examples (10-module-system)
module-system:
	@echo "=== Running Module System Examples ==="
	cargo run --example module_demo
	cargo run --example smart_home_modules

# Individual example targets
fraud_detection:
	cargo run --example fraud_detection

grule_demo:
	cargo run --example grule_demo

simple_query_demo:
	cargo run --features backward-chaining --example simple_query_demo

medical_diagnosis_demo:
	cargo run --features backward-chaining --example medical_diagnosis_demo

detective_system_demo:
	cargo run --features backward-chaining --example detective_system_demo

multiple_solutions_demo:
	cargo run --features backward-chaining --example multiple_solutions_demo

grl_query_demo:
	cargo run --features backward-chaining --example grl_query_demo

unification_demo:
	cargo run --features backward-chaining --example unification_demo

ecommerce_approval_demo:
	cargo run --features backward-chaining --example ecommerce_approval_demo

purchasing_flow_demo:
	cargo run --features backward-chaining --example purchasing_flow_demo

loan_approval_demo:
	cargo run --features backward-chaining --example loan_approval_demo

family_relations_demo:
	cargo run --features backward-chaining --example family_relations_demo

product_recommendation_demo:
	cargo run --features backward-chaining --example product_recommendation_demo

rete_index_demo:
	cargo run --features backward-chaining --example rete_index_demo

comprehensive_backward_test:
	cargo run --features backward-chaining --example comprehensive_backward_test

backward_edge_cases_test:
	cargo run --features backward-chaining --example backward_edge_cases_test

module_demo:
	cargo run --example module_demo

backward_critical_missing_tests:
	cargo run --features backward-chaining --example backward_critical_missing_tests

smart_home_modules:
	cargo run --example smart_home_modules

advanced_method_calls:
	cargo run --example advanced_method_calls

expression_demo:
	cargo run --example expression_demo

generic_method_calls:
	cargo run --example generic_method_calls

inline_rules_demo:
	cargo run --example inline_rules_demo

method_calls_demo:
	cargo run --example method_calls_demo

simple_pattern_matching_grl:
	cargo run --example simple_pattern_matching_grl

multifield_demo:
	cargo run --example multifield_demo

rete_call_function_demo:
	cargo run --example rete_call_function_demo

rete_deffacts_demo:
	cargo run --example rete_deffacts_demo

rete_demo:
	cargo run --example rete_demo

rete_grl_demo:
	cargo run --example rete_grl_demo

rete_memoization_demo:
	cargo run --example rete_memoization_demo

rete_multifield_demo:
	cargo run --example rete_multifield_demo

rete_parse_demo:
	cargo run --example rete_parse_demo

rete_template_globals_demo:
	cargo run --example rete_template_globals_demo

rete_typed_facts_demo:
	cargo run --example rete_typed_facts_demo

tms_demo:
	cargo run --example tms_demo

accumulate_demo:
	cargo run --example accumulate_demo

accumulate_grl_demo:
	cargo run --example accumulate_grl_demo

action_handlers_demo:
	cargo run --example action_handlers_demo

action_handlers_grl_demo:
	cargo run --example action_handlers_grl_demo

conflict_resolution_demo:
	cargo run --example conflict_resolution_demo

custom_functions_demo:
	cargo run --example custom_functions_demo

grl_no_loop_demo:
	cargo run --example grl_no_loop_demo

multifield_operations_demo:
	cargo run --example multifield_operations_demo

no_loop_demo:
	cargo run --example no_loop_demo

pattern_matching_from_grl:
	cargo run --example pattern_matching_from_grl

retract_demo_rete:
	cargo run --example retract_demo_rete

retract_demo:
	cargo run --example retract_demo

rule_attributes_demo:
	cargo run --example rule_attributes_demo

rule_templates_demo:
	cargo run --example rule_templates_demo

advanced_plugins_showcase:
	cargo run --example advanced_plugins_showcase

builtin_plugins_demo:
	cargo run --example builtin_plugins_demo

plugin_system_demo:
	cargo run --example plugin_system_demo

complete_speedup_demo:
	cargo run --example complete_speedup_demo

distributed_demo:
	cargo run --example distributed_demo

distributed_vs_single_demo:
	cargo run --example distributed_vs_single_demo

financial_stress_test:
	cargo run --example financial_stress_test

parallel_advanced_features_test:
	cargo run --example parallel_advanced_features_test

parallel_conditions_test:
	cargo run --example parallel_conditions_test

parallel_engine_demo:
	cargo run --example parallel_engine_demo

parallel_performance_demo:
	cargo run --example parallel_performance_demo

purchasing_rules_parse_benchmark:
	cargo run --example purchasing_rules_parse_benchmark

purchasing_rules_performance:
	cargo run --example purchasing_rules_performance

quick_engine_comparison:
	cargo run --example quick_engine_comparison

advanced_workflow_demo:
	cargo run --example advanced_workflow_demo

analytics_demo:
	cargo run --example analytics_demo

workflow_engine_demo:
	cargo run --example workflow_engine_demo

accumulate_rete_integration:
	cargo run --example accumulate_rete_integration

rete_engine_cached:
	cargo run --example rete_engine_cached

rete_p2_advanced_agenda:
	cargo run --example rete_p2_advanced_agenda

rete_p2_working_memory:
	cargo run --example rete_p2_working_memory

rete_p3_incremental:
	cargo run --example rete_p3_incremental

rete_p3_variable_binding:
	cargo run --example rete_p3_variable_binding

rete_ul_drools_style:
	cargo run --example rete_ul_drools_style

rule_dependency_analysis:
	cargo run --example rule_dependency_analysis

rule_file_functions_demo:
	cargo run --example rule_file_functions_demo

access_control_demo:
	cargo run --features backward-chaining --example access_control_demo

aggregation_demo:
	cargo run --features backward-chaining --example aggregation_demo

grl_aggregation_demo:
	cargo run --features backward-chaining --example grl_aggregation_demo
