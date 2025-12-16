//! Integration Tests for Stream Processing
//!
//! Real-world scenarios testing the complete flow:
//! GRL Parsing → StreamAlphaNode → Event Processing

#[cfg(feature = "streaming")]
mod stream_integration_tests {
    use rust_rule_engine::parser::grl::stream_syntax::parse_stream_pattern;
    use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
    use rust_rule_engine::streaming::event::StreamEvent;
    use rust_rule_engine::types::Value;
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn current_time_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    // ========================================================================
    // SCENARIO 1: E-Commerce Fraud Detection
    // ========================================================================

    #[test]
    fn test_fraud_detection_suspicious_ip_change() {
        println!("\n=== Fraud Detection: IP Change ===");

        // Parse GRL pattern
        let grl = r#"login: LoginEvent from stream("logins") over window(10 min, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).expect("Failed to parse GRL");

        // Create StreamAlphaNode from parsed pattern
        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type.clone(),
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // Scenario: User logs in from different IPs within 10 minutes
        let events = vec![
            // Login 1: New York (9 minutes ago)
            create_login_event(
                "user123",
                "192.168.1.1",
                "New York",
                current_time - 540_000,
            ),
            // Login 2: London (1 minute ago) - SUSPICIOUS!
            create_login_event(
                "user123",
                "10.0.0.1",
                "London",
                current_time - 60_000,
            ),
            // Login 3: Different user, same IP (30 seconds ago) - OK
            create_login_event(
                "user456",
                "192.168.1.1",
                "New York",
                current_time - 30_000,
            ),
        ];

        let mut accepted_events = Vec::new();
        for event in events {
            if node.process_event(&event) {
                accepted_events.push(event.clone());
            }
        }

        // All LoginEvents should be accepted
        assert_eq!(accepted_events.len(), 3);

        // Check we can detect IP changes
        let events = node.get_events();
        let user123_events: Vec<_> = events
            .iter()
            .filter(|e| e.get_string("user_id") == Some("user123"))
            .collect();

        assert_eq!(user123_events.len(), 2);

        // Detect IP change
        let ip1 = user123_events[0].get_string("ip_address").unwrap();
        let ip2 = user123_events[1].get_string("ip_address").unwrap();
        assert_ne!(ip1, ip2, "Should detect IP change");

        println!("✓ Detected suspicious IP change: {} -> {}", ip1, ip2);
        println!("✓ Events in window: {}", node.event_count());
    }

    #[test]
    fn test_fraud_detection_velocity_check() {
        println!("\n=== Fraud Detection: Purchase Velocity ===");

        let grl = r#"purchase: PurchaseEvent from stream("purchases") over window(5 min, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // Scenario: User makes 5 purchases in 3 minutes (suspicious!)
        for i in 0..5 {
            let event = create_purchase_event(
                "user789",
                99.99 + (i as f64),
                current_time - (i * 30_000), // Every 30 seconds
            );
            assert!(node.process_event(&event));
        }

        // Check velocity
        let purchases_in_window = node.event_count();
        assert_eq!(purchases_in_window, 5);

        // Velocity rule: More than 3 purchases in 5 minutes = suspicious
        let is_suspicious = purchases_in_window > 3;
        assert!(is_suspicious, "Should detect high velocity");

        println!("✓ Detected {} purchases in 5-minute window", purchases_in_window);
        println!("✓ Velocity check: FLAGGED");
    }

    // ========================================================================
    // SCENARIO 2: IoT Sensor Monitoring
    // ========================================================================

    #[test]
    fn test_iot_temperature_spike_detection() {
        println!("\n=== IoT: Temperature Spike Detection ===");

        let grl = r#"reading: TempReading from stream("sensors") over window(30 sec, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // Scenario: Temperature rises from 20°C to 85°C in 30 seconds
        let temperatures = vec![20.0, 25.0, 35.0, 50.0, 70.0, 85.0];

        for (i, temp) in temperatures.iter().enumerate() {
            let event = create_temp_reading(
                "sensor-001",
                *temp,
                current_time - ((temperatures.len() - i - 1) as u64 * 5_000),
            );
            node.process_event(&event);
        }

        // Analyze temperature trend
        let readings = node.get_events();
        let temps: Vec<f64> = readings
            .iter()
            .filter_map(|e| e.get_numeric("temperature"))
            .collect();

        assert_eq!(temps.len(), 6);

        let min_temp = temps.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_temp = temps.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let temp_delta = max_temp - min_temp;

        // Alert if temperature rises more than 50°C in 30 seconds
        let spike_detected = temp_delta > 50.0;
        assert!(spike_detected, "Should detect temperature spike");

        println!("✓ Temperature range: {:.1}°C - {:.1}°C", min_temp, max_temp);
        println!("✓ Delta: {:.1}°C (ALERT!)", temp_delta);
    }

    #[test]
    fn test_iot_tumbling_window_aggregation() {
        println!("\n=== IoT: Tumbling Window Aggregation ===");

        let grl = r#"metric: MetricEvent from stream("metrics") over window(10 sec, tumbling)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();
        let window_duration_ms = 10_000u64;
        let current_window_start = (current_time / window_duration_ms) * window_duration_ms;

        // Send events in current window
        for i in 0..5 {
            let event = create_metric_event(
                "cpu_usage",
                50.0 + (i as f64 * 5.0),
                current_window_start + (i * 1_000),
            );
            assert!(node.process_event(&event));
        }

        // Event from previous window should be rejected
        let old_event = create_metric_event(
            "cpu_usage",
            99.0,
            current_window_start - 5_000,
        );
        assert!(!node.process_event(&old_event));

        // Only current window events
        assert_eq!(node.event_count(), 5);

        println!("✓ Tumbling window contains {} events", node.event_count());
        println!("✓ Old events rejected correctly");
    }

    // ========================================================================
    // SCENARIO 3: Financial Trading
    // ========================================================================

    #[test]
    fn test_trading_momentum_detection() {
        println!("\n=== Trading: Momentum Detection ===");

        let grl = r#"tick: PriceTick from stream("market-data") over window(1 min, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // Stock price rises from $100 to $110 in 1 minute
        let prices = vec![100.0, 102.0, 104.5, 106.0, 108.0, 110.0];

        for (i, price) in prices.iter().enumerate() {
            let event = create_price_tick(
                "AAPL",
                *price,
                current_time - ((prices.len() - i - 1) as u64 * 10_000),
            );
            node.process_event(&event);
        }

        // Calculate momentum
        let ticks = node.get_events();
        let prices_in_window: Vec<f64> = ticks
            .iter()
            .filter_map(|e| e.get_numeric("price"))
            .collect();

        let first_price = prices_in_window.first().unwrap();
        let last_price = prices_in_window.last().unwrap();
        let price_change_percent = ((last_price - first_price) / first_price) * 100.0;

        // Momentum rule: > 5% change in 1 minute
        let strong_momentum = price_change_percent.abs() > 5.0;
        assert!(strong_momentum, "Should detect strong momentum");

        println!("✓ Price movement: ${:.2} -> ${:.2}", first_price, last_price);
        println!("✓ Change: {:.2}% (Strong momentum!)", price_change_percent);
    }

    // ========================================================================
    // SCENARIO 4: Security Intrusion Detection
    // ========================================================================

    #[test]
    fn test_security_brute_force_detection() {
        println!("\n=== Security: Brute Force Detection ===");

        let grl = r#"attempt: LoginAttempt from stream("auth-logs") over window(2 min, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // 10 failed login attempts in 2 minutes
        for i in 0..10 {
            let event = create_login_attempt(
                "admin",
                "192.168.1.100",
                false,
                current_time - (i * 10_000),
            );
            node.process_event(&event);
        }

        // Count failed attempts
        let attempts = node.get_events();
        let failed_count = attempts
            .iter()
            .filter(|e| e.get_boolean("success") == Some(false))
            .count();

        // Brute force rule: > 5 failed attempts in 2 minutes
        let brute_force_detected = failed_count > 5;
        assert!(brute_force_detected, "Should detect brute force");

        println!("✓ Failed attempts in window: {}", failed_count);
        println!("✓ Brute force attack detected!");
    }

    #[test]
    fn test_security_port_scanning_detection() {
        println!("\n=== Security: Port Scanning Detection ===");

        let grl = r#"conn: Connection from stream("network-events") over window(30 sec, sliding)"#;
        let (_, pattern) = parse_stream_pattern(grl).unwrap();

        let mut node = StreamAlphaNode::new(
            &pattern.source.stream_name,
            pattern.event_type,
            pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // Same IP scanning multiple ports
        let ports = vec![22, 23, 80, 443, 3306, 5432, 8080, 8443];
        for (i, port) in ports.iter().enumerate() {
            let event = create_connection_event(
                "192.168.1.200",
                *port,
                current_time - (i as u64 * 3_000),
            );
            node.process_event(&event);
        }

        // Count unique ports accessed
        let connections = node.get_events();
        let unique_ports: std::collections::HashSet<i64> = connections
            .iter()
            .filter_map(|e| match e.data.get("port") {
                Some(Value::Integer(p)) => Some(*p),
                _ => None,
            })
            .collect();

        // Port scan rule: > 5 different ports in 30 seconds
        let port_scan_detected = unique_ports.len() > 5;
        assert!(port_scan_detected, "Should detect port scanning");

        println!("✓ Unique ports accessed: {}", unique_ports.len());
        println!("✓ Ports: {:?}", unique_ports);
        println!("✓ Port scanning detected!");
    }

    // ========================================================================
    // SCENARIO 5: Multi-Stream Correlation (Advanced)
    // ========================================================================

    #[test]
    fn test_multi_stream_correlation() {
        println!("\n=== Multi-Stream: Login + Purchase Correlation ===");

        // Two separate streams
        let login_grl = r#"login: LoginEvent from stream("logins") over window(5 min, sliding)"#;
        let purchase_grl = r#"purchase: PurchaseEvent from stream("purchases") over window(5 min, sliding)"#;

        let (_, login_pattern) = parse_stream_pattern(login_grl).unwrap();
        let (_, purchase_pattern) = parse_stream_pattern(purchase_grl).unwrap();

        let mut login_node = StreamAlphaNode::new(
            &login_pattern.source.stream_name,
            login_pattern.event_type,
            login_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let mut purchase_node = StreamAlphaNode::new(
            &purchase_pattern.source.stream_name,
            purchase_pattern.event_type,
            purchase_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let current_time = current_time_ms();

        // User logs in from IP A
        let login = create_login_event("user999", "203.0.113.10", "USA", current_time - 60_000);
        let login_accepted = login_node.process_event(&login);
        println!("Login event accepted: {}", login_accepted);

        // Same user purchases from IP B (different!)
        let purchase = create_purchase_event("user999", 9999.99, current_time - 30_000);
        let purchase_accepted = purchase_node.process_event(&purchase);
        println!("Purchase event accepted: {}", purchase_accepted);

        // Correlate: Find if user_id exists in both streams
        let login_events = login_node.get_events();
        let purchase_events = purchase_node.get_events();

        println!("Login events count: {}", login_events.len());
        println!("Purchase events count: {}", purchase_events.len());

        let user_in_logins = login_events
            .iter()
            .any(|e| e.get_string("user_id") == Some("user999"));

        let user_in_purchases = purchase_events
            .iter()
            .any(|e| e.get_string("user_id") == Some("user999"));

        println!("User in logins: {}", user_in_logins);
        println!("User in purchases: {}", user_in_purchases);

        assert!(user_in_logins && user_in_purchases, "Should find user in both streams");

        println!("✓ User found in login stream: {}", user_in_logins);
        println!("✓ User found in purchase stream: {}", user_in_purchases);
        println!("✓ Cross-stream correlation successful!");
    }

    // ========================================================================
    // Helper Functions to Create Events
    // ========================================================================

    fn create_login_event(user_id: &str, ip: &str, location: &str, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("ip_address".to_string(), Value::String(ip.to_string()));
        data.insert("location".to_string(), Value::String(location.to_string()));

        StreamEvent::with_timestamp("LoginEvent", data, "logins", timestamp)
    }

    fn create_purchase_event(user_id: &str, amount: f64, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("amount".to_string(), Value::Number(amount));

        StreamEvent::with_timestamp("PurchaseEvent", data, "purchases", timestamp)
    }

    fn create_temp_reading(sensor_id: &str, temperature: f64, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("sensor_id".to_string(), Value::String(sensor_id.to_string()));
        data.insert("temperature".to_string(), Value::Number(temperature));

        StreamEvent::with_timestamp("TempReading", data, "sensors", timestamp)
    }

    fn create_metric_event(metric_name: &str, value: f64, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("metric_name".to_string(), Value::String(metric_name.to_string()));
        data.insert("value".to_string(), Value::Number(value));

        StreamEvent::with_timestamp("MetricEvent", data, "metrics", timestamp)
    }

    fn create_price_tick(symbol: &str, price: f64, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("symbol".to_string(), Value::String(symbol.to_string()));
        data.insert("price".to_string(), Value::Number(price));

        StreamEvent::with_timestamp("PriceTick", data, "market-data", timestamp)
    }

    fn create_login_attempt(
        username: &str,
        ip: &str,
        success: bool,
        timestamp: u64,
    ) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("username".to_string(), Value::String(username.to_string()));
        data.insert("ip_address".to_string(), Value::String(ip.to_string()));
        data.insert("success".to_string(), Value::Boolean(success));

        StreamEvent::with_timestamp("LoginAttempt", data, "auth-logs", timestamp)
    }

    fn create_connection_event(source_ip: &str, port: i64, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("source_ip".to_string(), Value::String(source_ip.to_string()));
        data.insert("port".to_string(), Value::Integer(port));

        StreamEvent::with_timestamp("Connection", data, "network-events", timestamp)
    }
}
