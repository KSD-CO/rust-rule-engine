use rust_rule_engine::*;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

/// 🌐 Demo: So sánh Single vs Distributed Processing
/// 
/// Ví dụ này cho thấy sự khác biệt giữa:
/// 1. Xử lý tập trung (1 engine làm tất cả)
/// 2. Xử lý phân tán (nhiều engine làm song song)

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌐 === DISTRIBUTED vs SINGLE NODE DEMO ===");
    println!("So sánh hiệu suất giữa xử lý tập trung và phân tán\n");

    // Tạo dữ liệu test với nhiều đơn hàng
    let orders = create_multiple_orders()?;
    println!("📦 Tạo {} đơn hàng để xử lý", orders.len());

    // Demo 1: Single Node Processing (1 engine làm tất cả)
    println!("\n🔄 Demo 1: SINGLE NODE PROCESSING");
    println!("   1 engine xử lý tất cả {} đơn hàng", orders.len());
    let single_start = Instant::now();
    let single_results = process_single_node(&orders)?;
    let single_duration = single_start.elapsed();
    
    println!("   ✅ Hoàn thành trong: {:?}", single_duration);
    println!("   📊 Tổng rules fired: {}", single_results.iter().sum::<usize>());

    // Demo 2: Distributed Processing (nhiều engine song song)
    println!("\n🌐 Demo 2: DISTRIBUTED PROCESSING");
    println!("   3 engines xử lý song song {} đơn hàng", orders.len());
    let distributed_start = Instant::now();
    let distributed_results = process_distributed(&orders)?;
    let distributed_duration = distributed_start.elapsed();
    
    println!("   ✅ Hoàn thành trong: {:?}", distributed_duration);
    println!("   📊 Tổng rules fired: {}", distributed_results.iter().sum::<usize>());

    // So sánh kết quả
    println!("\n📈 KẾT QUẢ SO SÁNH:");
    println!("   Single Node:    {:>8.2?}", single_duration);
    println!("   Distributed:    {:>8.2?}", distributed_duration);
    
    if distributed_duration < single_duration {
        let speedup = single_duration.as_secs_f64() / distributed_duration.as_secs_f64();
        println!("   🚀 Tăng tốc:     {:>8.1}x", speedup);
        println!("   💡 Distributed nhanh hơn!");
    } else {
        println!("   ⚠️ Single node vẫn nhanh hơn (do overhead)");
    }

    println!("\n🎯 Giải thích:");
    println!("   📍 Single Node: 1 engine xử lý tuần tự từng đơn hàng");
    println!("   🌐 Distributed: 3 engines xử lý song song, chia đều công việc");
    println!("   ⚡ Khi có nhiều đơn hàng, distributed sẽ nhanh hơn rõ rệt");

    Ok(())
}

/// Tạo nhiều đơn hàng để test
fn create_multiple_orders() -> std::result::Result<Vec<Facts>, Box<dyn std::error::Error>> {
    let mut orders = Vec::new();
    
    // Tạo 12 đơn hàng khác nhau
    let order_data = vec![
        ("ORD-001", "John Doe", 1500.0, 2),
        ("ORD-002", "Jane Smith", 800.0, 1),
        ("ORD-003", "Bob Johnson", 2200.0, 3),
        ("ORD-004", "Alice Brown", 650.0, 1),
        ("ORD-005", "Charlie Wilson", 1800.0, 2),
        ("ORD-006", "Diana Lee", 950.0, 1),
        ("ORD-007", "Eve Davis", 3000.0, 4),
        ("ORD-008", "Frank Miller", 1200.0, 2),
        ("ORD-009", "Grace Taylor", 750.0, 1),
        ("ORD-010", "Henry Clark", 2500.0, 3),
        ("ORD-011", "Ivy Anderson", 1100.0, 2),
        ("ORD-012", "Jack White", 1750.0, 2),
    ];

    for (order_id, customer_name, amount, item_count) in order_data {
        let facts = Facts::new();
        
        // Order info
        let mut order_props = HashMap::new();
        order_props.insert("OrderID".to_string(), Value::String(order_id.to_string()));
        order_props.insert("CustomerName".to_string(), Value::String(customer_name.to_string()));
        order_props.insert("TotalAmount".to_string(), Value::Number(amount));
        order_props.insert("ItemCount".to_string(), Value::Integer(item_count));
        order_props.insert("Status".to_string(), Value::String("PENDING".to_string()));
        order_props.insert("Priority".to_string(), Value::String("NORMAL".to_string()));
        order_props.insert("ProcessingFee".to_string(), Value::Number(0.0));
        facts.add_value("Order", Value::Object(order_props))?;

        // Customer info
        let mut customer_props = HashMap::new();
        customer_props.insert("Name".to_string(), Value::String(customer_name.to_string()));
        customer_props.insert("Tier".to_string(), Value::String(if amount > 2000.0 { "VIP" } else { "STANDARD" }.to_string()));
        customer_props.insert("TotalSpent".to_string(), Value::Number(amount * 2.0)); // Giả sử đã mua trước đó
        facts.add_value("Customer", Value::Object(customer_props))?;

        orders.push(facts);
    }

    Ok(orders)
}

/// Xử lý single node (1 engine làm tất cả)
fn process_single_node(orders: &[Facts]) -> std::result::Result<Vec<usize>, Box<dyn std::error::Error>> {
    println!("   🔨 Tạo 1 engine duy nhất...");
    
    // Tạo 1 engine với tất cả rules
    let mut engine = create_full_engine("SingleNode")?;
    
    let mut results = Vec::new();
    
    // Xử lý từng đơn hàng tuần tự
    for (i, order) in orders.iter().enumerate() {
        print!("   📦 Xử lý đơn hàng {}...", i + 1);
        let result = engine.execute(order)?;
        results.push(result.rules_fired);
        println!(" {} rules fired", result.rules_fired);
        
        // Giả lập thời gian xử lý
        thread::sleep(Duration::from_millis(100));
    }
    
    Ok(results)
}

/// Xử lý distributed (nhiều engines song song)
fn process_distributed(orders: &[Facts]) -> std::result::Result<Vec<usize>, Box<dyn std::error::Error>> {
    println!("   🌐 Tạo 3 engines phân tán...");
    
    // Chia đơn hàng thành 3 nhóm
    let chunk_size = (orders.len() + 2) / 3; // Chia đều cho 3 engines
    let chunks: Vec<&[Facts]> = orders.chunks(chunk_size).collect();
    
    println!("   📊 Phân chia: {} + {} + {} đơn hàng", 
        chunks.get(0).map(|c| c.len()).unwrap_or(0),
        chunks.get(1).map(|c| c.len()).unwrap_or(0),
        chunks.get(2).map(|c| c.len()).unwrap_or(0)
    );

    // Xử lý song song bằng threads
    let mut handles = Vec::new();
    
    for (worker_id, chunk) in chunks.into_iter().enumerate() {
        let chunk_owned: Vec<Facts> = chunk.iter().cloned().collect();
        
        let handle = thread::spawn(move || -> std::result::Result<Vec<usize>, Box<dyn std::error::Error>> {
            println!("   🔨 Worker {} bắt đầu với {} đơn hàng", worker_id + 1, chunk_owned.len());
            
            // Mỗi worker có engine riêng
            let mut engine = create_full_engine(&format!("Worker{}", worker_id + 1))?;
            let mut worker_results = Vec::new();
            
            for (i, order) in chunk_owned.iter().enumerate() {
                let result = engine.execute(order)?;
                worker_results.push(result.rules_fired);
                println!("   ⚡ Worker {} - Đơn hàng {}: {} rules", worker_id + 1, i + 1, result.rules_fired);
                
                // Giả lập thời gian xử lý (ngắn hơn vì song song)
                thread::sleep(Duration::from_millis(50));
            }
            
            println!("   ✅ Worker {} hoàn thành!", worker_id + 1);
            Ok(worker_results)
        });
        
        handles.push(handle);
    }
    
    // Chờ tất cả workers hoàn thành
    let mut all_results = Vec::new();
    for handle in handles {
        let worker_results = handle.join().map_err(|_| "Thread join error")??;
        all_results.extend(worker_results);
    }
    
    Ok(all_results)
}

/// Tạo engine với đầy đủ rules
fn create_full_engine(name: &str) -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new(name);
    let config = EngineConfig {
        max_cycles: 3,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Thêm rules xử lý đơn hàng
    let rules = vec![
        // Rule 1: VIP customer priority
        r#"rule "VIPPriority" salience 20 {
            when Customer.Tier == "VIP" && Order.Status == "PENDING"
            then Order.Priority = "HIGH";
        }"#,
        
        // Rule 2: Large order processing fee
        r#"rule "LargeOrderFee" salience 15 {
            when Order.TotalAmount > 1500.0
            then Order.ProcessingFee = Order.TotalAmount * 0.02;
        }"#,
        
        // Rule 3: Standard processing fee  
        r#"rule "StandardFee" salience 10 {
            when Order.TotalAmount <= 1500.0
            then Order.ProcessingFee = 25.0;
        }"#,
        
        // Rule 4: High priority processing
        r#"rule "HighPriorityProcessing" salience 5 {
            when Order.Priority == "HIGH"
            then Order.Status = "PRIORITY_QUEUE";
        }"#,
        
        // Rule 5: Standard processing
        r#"rule "StandardProcessing" salience 3 {
            when Order.Priority == "NORMAL" && Order.Status == "PENDING"
            then Order.Status = "PROCESSING";
        }"#,
        
        // Rule 6: Multi-item discount
        r#"rule "MultiItemDiscount" salience 8 {
            when Order.ItemCount > 2
            then Order.ProcessingFee = Order.ProcessingFee * 0.9;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.add_rule(rule)?;
        }
    }

    Ok(engine)
}
