//! Complete Streaming Example: Real-Time Fraud Detection
//!
//! This example demonstrates a production-ready streaming rule engine:
//! - Real-time event processing from multiple streams
//! - Complex fraud detection rules
//! - Window-based aggregations
//! - Cross-stream correlations
//! - Rule evaluation on streaming data

use rust_rule_engine::parser::grl::stream_syntax::parse_stream_pattern;
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
use rust_rule_engine::rete::working_memory::WorkingMemory;
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Simulated fraud detection system
struct FraudDetectionSystem {
    // Stream processors
    login_stream: StreamAlphaNode,
    purchase_stream: StreamAlphaNode,
    location_stream: StreamAlphaNode,

    // Working memory (RETE network)
    working_memory: WorkingMemory,

    // Metrics
    total_events: usize,
    alerts_triggered: usize,
}

impl FraudDetectionSystem {
    fn new() -> Self {
        println!("🚀 Initializing Fraud Detection System...\n");

        // Parse GRL patterns for each stream
        let login_grl = r#"login: LoginEvent from stream("logins") over window(15 min, sliding)"#;
        let purchase_grl = r#"purchase: PurchaseEvent from stream("purchases") over window(15 min, sliding)"#;
        let location_grl = r#"location: LocationEvent from stream("locations") over window(30 min, sliding)"#;

        let (_, login_pattern) = parse_stream_pattern(login_grl).unwrap();
        let (_, purchase_pattern) = parse_stream_pattern(purchase_grl).unwrap();
        let (_, location_pattern) = parse_stream_pattern(location_grl).unwrap();

        println!("✓ Parsed stream patterns:");
        println!("  - Login stream (15 min sliding window)");
        println!("  - Purchase stream (15 min sliding window)");
        println!("  - Location stream (30 min sliding window)\n");

        // Create stream processors
        let login_stream = StreamAlphaNode::new(
            &login_pattern.source.stream_name,
            login_pattern.event_type,
            login_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let purchase_stream = StreamAlphaNode::new(
            &purchase_pattern.source.stream_name,
            purchase_pattern.event_type,
            purchase_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        let location_stream = StreamAlphaNode::new(
            &location_pattern.source.stream_name,
            location_pattern.event_type,
            location_pattern.source.window.as_ref().map(|w| WindowSpec {
                duration: w.duration,
                window_type: w.window_type.clone(),
            }),
        );

        Self {
            login_stream,
            purchase_stream,
            location_stream,
            working_memory: WorkingMemory::new(),
            total_events: 0,
            alerts_triggered: 0,
        }
    }

    /// Process login event
    fn process_login(&mut self, user_id: &str, ip: &str, location: &str, device: &str) {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("ip_address".to_string(), Value::String(ip.to_string()));
        data.insert("location".to_string(), Value::String(location.to_string()));
        data.insert("device".to_string(), Value::String(device.to_string()));

        let event = StreamEvent::new("LoginEvent", data, "logins");

        if self.login_stream.process_event(&event) {
            self.working_memory.insert_from_stream("logins".to_string(), event);
            self.total_events += 1;
        }
    }

    /// Process purchase event
    fn process_purchase(&mut self, user_id: &str, amount: f64, ip: &str, merchant: &str) {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("amount".to_string(), Value::Number(amount));
        data.insert("ip_address".to_string(), Value::String(ip.to_string()));
        data.insert("merchant".to_string(), Value::String(merchant.to_string()));

        let event = StreamEvent::new("PurchaseEvent", data, "purchases");

        if self.purchase_stream.process_event(&event) {
            self.working_memory.insert_from_stream("purchases".to_string(), event);
            self.total_events += 1;
        }
    }

    /// Process location change event
    fn process_location_change(&mut self, user_id: &str, old_location: &str, new_location: &str) {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("old_location".to_string(), Value::String(old_location.to_string()));
        data.insert("new_location".to_string(), Value::String(new_location.to_string()));

        let event = StreamEvent::new("LocationEvent", data, "locations");

        if self.location_stream.process_event(&event) {
            self.working_memory.insert_from_stream("locations".to_string(), event);
            self.total_events += 1;
        }
    }

    /// Rule 1: Detect IP address changes during login
    fn check_suspicious_ip_change(&mut self) {
        let logins = self.login_stream.get_events();

        // Group by user_id
        let mut user_logins: HashMap<String, Vec<&StreamEvent>> = HashMap::new();
        for login in logins.iter() {
            if let Some(user_id) = login.get_string("user_id") {
                user_logins.entry(user_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(login);
            }
        }

        // Collect alerts first (avoid borrow checker issue)
        let mut alerts = Vec::new();
        for (user_id, events) in user_logins {
            if events.len() >= 2 {
                let mut ips: Vec<String> = events.iter()
                    .filter_map(|e| e.get_string("ip_address").map(|s| s.to_string()))
                    .collect();
                ips.sort();
                ips.dedup();

                if ips.len() > 1 {
                    alerts.push((user_id, format!("Multiple IPs detected: {:?}", ips)));
                }
            }
        }

        // Trigger alerts after dropping immutable borrow
        for (user_id, details) in alerts {
            self.trigger_alert("SUSPICIOUS_IP_CHANGE", &user_id, &details, "HIGH");
        }
    }

    /// Rule 2: Detect high velocity purchases
    fn check_purchase_velocity(&mut self) {
        let purchases = self.purchase_stream.get_events();

        // Group by user_id
        let mut user_purchases: HashMap<String, Vec<&StreamEvent>> = HashMap::new();
        for purchase in purchases.iter() {
            if let Some(user_id) = purchase.get_string("user_id") {
                user_purchases.entry(user_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(purchase);
            }
        }

        // Collect alerts first (avoid borrow checker issue)
        let mut alerts = Vec::new();
        for (user_id, events) in user_purchases {
            if events.len() >= 3 {
                // Calculate total amount
                let total: f64 = events.iter()
                    .filter_map(|e| e.get_numeric("amount"))
                    .sum();

                let severity = if events.len() > 5 { "CRITICAL" } else { "MEDIUM" };
                alerts.push((
                    user_id,
                    format!("{} purchases totaling ${:.2} in 15 minutes", events.len(), total),
                    severity
                ));
            }
        }

        // Trigger alerts after dropping immutable borrow
        for (user_id, details, severity) in alerts {
            self.trigger_alert("HIGH_VELOCITY_PURCHASES", &user_id, &details, severity);
        }
    }

    /// Rule 3: Detect impossible travel (login from different locations too quickly)
    fn check_impossible_travel(&mut self) {
        let logins = self.login_stream.get_events();

        // Group by user_id
        let mut user_logins: HashMap<String, Vec<&StreamEvent>> = HashMap::new();
        for login in logins.iter() {
            if let Some(user_id) = login.get_string("user_id") {
                user_logins.entry(user_id.to_string())
                    .or_insert_with(Vec::new)
                    .push(login);
            }
        }

        // Collect alerts first (avoid borrow checker issue)
        let mut alerts = Vec::new();
        for (user_id, events) in user_logins {
            if events.len() >= 2 {
                let locations: Vec<String> = events.iter()
                    .filter_map(|e| e.get_string("location").map(|s| s.to_string()))
                    .collect();

                if locations.len() >= 2 && locations.first() != locations.last() {
                    let time_diff_ms = events.last().unwrap().metadata.timestamp
                        - events.first().unwrap().metadata.timestamp;
                    let time_diff_min = time_diff_ms / 60_000;

                    if time_diff_min < 60 {
                        alerts.push((
                            user_id,
                            format!("Travel from {} to {} in {} minutes",
                                locations.first().unwrap(),
                                locations.last().unwrap(),
                                time_diff_min)
                        ));
                    }
                }
            }
        }

        // Trigger alerts after dropping immutable borrow
        for (user_id, details) in alerts {
            self.trigger_alert("IMPOSSIBLE_TRAVEL", &user_id, &details, "CRITICAL");
        }
    }

    /// Rule 4: Detect purchase from different IP than login
    fn check_purchase_ip_mismatch(&mut self) {
        let logins = self.working_memory.get_by_type("logins");
        let purchases = self.working_memory.get_by_type("purchases");

        // Collect alerts first (avoid borrow checker issue)
        let mut alerts = Vec::new();
        for login in logins {
            if let Some(login_user) = login.data.get("user_id") {
                for purchase in &purchases {
                    if let Some(purchase_user) = purchase.data.get("user_id") {
                        if login_user == purchase_user {
                            let login_ip = login.data.get("ip_address");
                            let purchase_ip = purchase.data.get("ip_address");

                            if login_ip != purchase_ip {
                                alerts.push((
                                    format!("{:?}", login_user),
                                    format!("Login IP: {:?}, Purchase IP: {:?}", login_ip, purchase_ip)
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Trigger alerts after dropping immutable borrow
        for (user_id, details) in alerts {
            self.trigger_alert("IP_MISMATCH", &user_id, &details, "MEDIUM");
        }
    }

    /// Trigger fraud alert
    fn trigger_alert(&mut self, alert_type: &str, user_id: &str, details: &str, severity: &str) {
        self.alerts_triggered += 1;

        let emoji = match severity {
            "CRITICAL" => "🚨",
            "HIGH" => "⚠️",
            "MEDIUM" => "⚡",
            _ => "ℹ️",
        };

        println!("{} FRAUD ALERT [{}]", emoji, severity);
        println!("   Type: {}", alert_type);
        println!("   User: {}", user_id);
        println!("   Details: {}", details);
        println!();
    }

    /// Evaluate all fraud detection rules
    fn evaluate_rules(&mut self) {
        println!("🔍 Evaluating fraud detection rules...\n");

        self.check_suspicious_ip_change();
        self.check_purchase_velocity();
        self.check_impossible_travel();
        self.check_purchase_ip_mismatch();
    }

    /// Print system statistics
    fn print_stats(&self) {
        println!("\n📊 System Statistics");
        println!("═══════════════════════════════════");
        println!("Total events processed: {}", self.total_events);
        println!("Alerts triggered: {}", self.alerts_triggered);
        println!();

        println!("Stream Statistics:");
        let login_stats = self.login_stream.window_stats();
        let purchase_stats = self.purchase_stream.window_stats();
        let location_stats = self.location_stream.window_stats();

        println!("  Login stream: {} events in window", login_stats.event_count);
        println!("  Purchase stream: {} events in window", purchase_stats.event_count);
        println!("  Location stream: {} events in window", location_stats.event_count);
        println!();

        println!("Working Memory:");
        println!("  Total facts: {}", self.working_memory.get_all_facts().len());
        println!("  Login facts: {}", self.working_memory.get_by_type("logins").len());
        println!("  Purchase facts: {}", self.working_memory.get_by_type("purchases").len());
        println!("  Location facts: {}", self.working_memory.get_by_type("locations").len());
    }
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║  Real-Time Fraud Detection System                 ║");
    println!("║  Powered by Rust Rule Engine + Streaming          ║");
    println!("╚════════════════════════════════════════════════════╝\n");

    let mut system = FraudDetectionSystem::new();

    println!("📡 Simulating real-time event stream...\n");
    std::thread::sleep(Duration::from_millis(500));

    // Scenario 1: Normal user behavior
    println!("─── Scenario 1: Normal User (alice) ───");
    system.process_login("alice", "192.168.1.100", "New York", "iPhone");
    println!("✓ alice logged in from New York");

    std::thread::sleep(Duration::from_millis(100));
    system.process_purchase("alice", 49.99, "192.168.1.100", "Amazon");
    println!("✓ alice purchased $49.99 from same IP\n");

    // Scenario 2: Suspicious IP change
    println!("─── Scenario 2: Suspicious Activity (bob) ───");
    system.process_login("bob", "10.0.0.1", "San Francisco", "Chrome");
    println!("✓ bob logged in from San Francisco (10.0.0.1)");

    std::thread::sleep(Duration::from_millis(100));
    system.process_login("bob", "203.0.113.50", "London", "Chrome");
    println!("✓ bob logged in from London (203.0.113.50) - DIFFERENT IP!");

    std::thread::sleep(Duration::from_millis(100));
    system.process_purchase("bob", 9999.99, "203.0.113.50", "LuxuryGoods.com");
    println!("✓ bob purchased $9,999.99\n");

    // Scenario 3: High velocity purchases
    println!("─── Scenario 3: High Velocity (charlie) ───");
    system.process_login("charlie", "172.16.0.1", "Tokyo", "Safari");
    println!("✓ charlie logged in from Tokyo");

    for i in 1..=6 {
        std::thread::sleep(Duration::from_millis(50));
        system.process_purchase("charlie", 199.99, "172.16.0.1", &format!("Merchant{}", i));
        println!("✓ charlie purchased ${} (purchase #{})", 199.99, i);
    }
    println!();

    // Scenario 4: Impossible travel
    println!("─── Scenario 4: Impossible Travel (dave) ───");
    system.process_login("dave", "192.168.2.1", "Paris", "Firefox");
    println!("✓ dave logged in from Paris");

    std::thread::sleep(Duration::from_millis(200));
    system.process_login("dave", "192.168.2.2", "Sydney", "Firefox");
    println!("✓ dave logged in from Sydney 20 seconds later - IMPOSSIBLE!\n");

    // Scenario 5: IP mismatch
    println!("─── Scenario 5: IP Mismatch (eve) ───");
    system.process_login("eve", "10.1.1.1", "Berlin", "Edge");
    println!("✓ eve logged in from Berlin (10.1.1.1)");

    std::thread::sleep(Duration::from_millis(100));
    system.process_purchase("eve", 499.99, "10.2.2.2", "Electronics.com");
    println!("✓ eve purchased $499.99 from DIFFERENT IP (10.2.2.2)!\n");

    // Evaluate all fraud detection rules
    println!("\n═══════════════════════════════════════════════════════\n");
    system.evaluate_rules();

    // Print final statistics
    system.print_stats();

    println!("\n╔════════════════════════════════════════════════════╗");
    println!("║  Fraud Detection Complete                         ║");
    println!("╚════════════════════════════════════════════════════╝\n");
}
