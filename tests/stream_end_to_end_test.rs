//! End-to-End Test: GRL File → Parser → RETE → WorkingMemory
//!
//! This test demonstrates the complete flow of stream processing:
//! 1. Parse GRL stream patterns
//! 2. Create StreamAlphaNode from patterns
//! 3. Process stream events
//! 4. Insert matched events into WorkingMemory
//! 5. Query facts from WorkingMemory

#[cfg(feature = "streaming")]
mod end_to_end_tests {
    use rust_rule_engine::parser::grl::stream_syntax::parse_stream_pattern;
    use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
    use rust_rule_engine::rete::working_memory::WorkingMemory;
    use rust_rule_engine::streaming::event::StreamEvent;
    use rust_rule_engine::types::Value;
    use std::collections::HashMap;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn current_time_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn test_end_to_end_fraud_detection() {
        println!("\n=== End-to-End: Fraud Detection System ===\n");

        // Step 1: Parse GRL patterns
        println!("Step 1: Parsing GRL patterns...");
        let login_grl = r#"login: LoginEvent from stream("logins") over window(10 min, sliding)"#;
        let purchase_grl =
            r#"purchase: PurchaseEvent from stream("purchases") over window(10 min, sliding)"#;

        let (_, login_pattern) = parse_stream_pattern(login_grl).unwrap();
        let (_, purchase_pattern) = parse_stream_pattern(purchase_grl).unwrap();
        println!("  ✓ Parsed login pattern: {}", login_pattern.var_name);
        println!("  ✓ Parsed purchase pattern: {}", purchase_pattern.var_name);

        // Step 2: Create StreamAlphaNodes
        println!("\nStep 2: Creating StreamAlphaNodes...");
        let mut login_node = StreamAlphaNode::new(
            &login_pattern.source.stream_name,
            login_pattern.event_type.clone(),
            login_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let mut purchase_node = StreamAlphaNode::new(
            &purchase_pattern.source.stream_name,
            purchase_pattern.event_type.clone(),
            purchase_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );
        println!(
            "  ✓ Created login node for stream: {}",
            login_pattern.source.stream_name
        );
        println!(
            "  ✓ Created purchase node for stream: {}",
            purchase_pattern.source.stream_name
        );

        // Step 3: Initialize WorkingMemory
        println!("\nStep 3: Initializing WorkingMemory...");
        let mut wm = WorkingMemory::new();
        println!("  ✓ WorkingMemory initialized");

        // Step 4: Process stream events
        println!("\nStep 4: Processing stream events...");
        let current_time = current_time_ms();

        // User logs in from New York
        let mut login_data = HashMap::new();
        login_data.insert("user_id".to_string(), Value::String("user123".to_string()));
        login_data.insert(
            "ip_address".to_string(),
            Value::String("192.168.1.1".to_string()),
        );
        login_data.insert(
            "location".to_string(),
            Value::String("New York".to_string()),
        );
        let login_event =
            StreamEvent::with_timestamp("LoginEvent", login_data, "logins", current_time - 60_000);

        if login_node.process_event(&login_event) {
            let handle = wm.insert_from_stream("logins".to_string(), login_event.clone());
            println!("  ✓ Login event inserted into WorkingMemory: {}", handle);
        }

        // User makes a suspicious purchase from London
        let mut purchase_data = HashMap::new();
        purchase_data.insert("user_id".to_string(), Value::String("user123".to_string()));
        purchase_data.insert("amount".to_string(), Value::Number(9999.99));
        purchase_data.insert(
            "ip_address".to_string(),
            Value::String("10.0.0.1".to_string()),
        );
        purchase_data.insert("location".to_string(), Value::String("London".to_string()));
        let purchase_event = StreamEvent::with_timestamp(
            "PurchaseEvent",
            purchase_data,
            "purchases",
            current_time - 30_000,
        );

        if purchase_node.process_event(&purchase_event) {
            let handle = wm.insert_from_stream("purchases".to_string(), purchase_event.clone());
            println!("  ✓ Purchase event inserted into WorkingMemory: {}", handle);
        }

        // Step 5: Query WorkingMemory
        println!("\nStep 5: Querying WorkingMemory...");
        let all_facts = wm.get_all_facts();
        println!("  ✓ Total facts in WorkingMemory: {}", all_facts.len());

        let login_facts = wm.get_by_type("logins");
        let purchase_facts = wm.get_by_type("purchases");
        println!("  ✓ Login facts: {}", login_facts.len());
        println!("  ✓ Purchase facts: {}", purchase_facts.len());

        // Step 6: Fraud detection logic
        println!("\nStep 6: Fraud Detection Analysis...");

        // Check for same user in both streams
        let mut fraud_detected = false;
        for login_fact in login_facts {
            if let Some(login_user) = login_fact.data.get("user_id") {
                for purchase_fact in &purchase_facts {
                    if let Some(purchase_user) = purchase_fact.data.get("user_id") {
                        if login_user == purchase_user {
                            // Check if IPs are different
                            let login_ip = login_fact.data.get("ip_address");
                            let purchase_ip = purchase_fact.data.get("ip_address");

                            if login_ip != purchase_ip {
                                fraud_detected = true;
                                println!("  ⚠️  FRAUD ALERT!");
                                println!("     User: {:?}", login_user);
                                println!("     Login IP: {:?}", login_ip);
                                println!("     Purchase IP: {:?}", purchase_ip);

                                // Check stream metadata
                                if let Some(ref stream_name) = login_fact.stream_source {
                                    println!("     Login from stream: {}", stream_name);
                                }
                                if let Some(ref stream_name) = purchase_fact.stream_source {
                                    println!("     Purchase from stream: {}", stream_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        assert!(fraud_detected, "Should detect fraud");
        println!("\n  ✅ End-to-end test completed successfully!");
    }

    #[test]
    fn test_end_to_end_iot_monitoring() {
        println!("\n=== End-to-End: IoT Temperature Monitoring ===\n");

        // Parse GRL pattern
        let grl = r#"reading: TempReading from stream("sensors") over window(1 min, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();
        println!("✓ Parsed sensor pattern: {}", pattern.var_name);

        // Create node
        let mut sensor_node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        // Initialize WorkingMemory
        let mut wm = WorkingMemory::new();
        let current_time = current_time_ms();

        // Process temperature readings
        println!("\n📊 Processing temperature readings...");
        let temperatures = [20.0, 25.0, 35.0, 50.0, 70.0, 85.0];

        for (i, temp) in temperatures.iter().enumerate() {
            let mut data = HashMap::new();
            data.insert(
                "sensor_id".to_string(),
                Value::String("sensor-001".to_string()),
            );
            data.insert("temperature".to_string(), Value::Number(*temp));
            data.insert("reading_num".to_string(), Value::Integer(i as i64));

            let event = StreamEvent::with_timestamp(
                "TempReading",
                data,
                "sensors",
                current_time - ((temperatures.len() - i - 1) as u64 * 5_000),
            );

            if sensor_node.process_event(&event) {
                let handle = wm.insert_from_stream("sensors".to_string(), event);
                println!("  Reading {}: {:.1}°C → {}", i + 1, temp, handle);
            }
        }

        // Analyze temperature trend
        println!("\n🔍 Analyzing temperature trend...");
        let sensor_facts = wm.get_by_type("sensors");
        println!("  Facts in WorkingMemory: {}", sensor_facts.len());

        let mut temps: Vec<f64> = sensor_facts
            .iter()
            .filter_map(|f| match f.data.get("temperature")? {
                rust_rule_engine::rete::facts::FactValue::Float(t) => Some(*t),
                _ => None,
            })
            .collect();

        temps.sort_by(|a, b| a.partial_cmp(b).unwrap());

        if !temps.is_empty() {
            let min_temp = temps.first().unwrap();
            let max_temp = temps.last().unwrap();
            let delta = max_temp - min_temp;

            println!("  Min temperature: {:.1}°C", min_temp);
            println!("  Max temperature: {:.1}°C", max_temp);
            println!("  Delta: {:.1}°C", delta);

            if delta > 50.0 {
                println!("  🚨 ALERT: Temperature spike detected!");
            }

            assert!(delta > 50.0, "Should detect temperature spike");
        }

        println!("\n  ✅ IoT monitoring test completed!");
    }

    #[test]
    fn test_end_to_end_multi_window_types() {
        println!("\n=== End-to-End: Multiple Window Types ===\n");

        // Create two patterns with different window types
        let sliding_grl = r#"event: Event from stream("events") over window(30 sec, sliding)"#;
        let tumbling_grl = r#"event: Event from stream("events") over window(10 sec, tumbling)"#;

        let (_, sliding_pattern) = parse_stream_pattern(sliding_grl).unwrap();
        let (_, tumbling_pattern) = parse_stream_pattern(tumbling_grl).unwrap();

        let mut sliding_node = StreamAlphaNode::new(
            &sliding_pattern.source.stream_name,
            sliding_pattern.event_type,
            sliding_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let mut tumbling_node = StreamAlphaNode::new(
            &tumbling_pattern.source.stream_name,
            tumbling_pattern.event_type,
            tumbling_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let mut wm_sliding = WorkingMemory::new();
        let mut wm_tumbling = WorkingMemory::new();

        let current_time = current_time_ms();

        // Send 5 events
        println!("📤 Sending events...");
        for i in 0..5 {
            let mut data = HashMap::new();
            data.insert("value".to_string(), Value::Integer(i));

            let event = StreamEvent::with_timestamp(
                "Event",
                data,
                "events",
                current_time - (i as u64 * 5_000),
            );

            if sliding_node.process_event(&event) {
                wm_sliding.insert_from_stream("events".to_string(), event.clone());
            }

            if tumbling_node.process_event(&event) {
                wm_tumbling.insert_from_stream("events".to_string(), event.clone());
            }
        }

        let sliding_count = wm_sliding.get_by_type("events").len();
        let tumbling_count = wm_tumbling.get_by_type("events").len();

        println!("  Sliding window facts: {}", sliding_count);
        println!("  Tumbling window facts: {}", tumbling_count);

        // Both should have events (specific counts depend on timing)
        assert!(sliding_count > 0, "Sliding window should have events");
        assert!(tumbling_count > 0, "Tumbling window should have events");

        println!("\n  ✅ Multi-window test completed!");
    }
}
