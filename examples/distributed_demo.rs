use rust_rule_engine::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{sleep, Duration};

/// 🌐 Distributed Rule Engine - Proof of Concept
/// 
/// This demo shows how to implement basic distributed rule execution
/// across multiple "virtual nodes" in the same process.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: String,
    pub node_type: NodeType,
    pub status: NodeStatus,
    pub rules_count: usize,
    pub facts_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Master,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Busy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedTask {
    pub task_id: String,
    pub rules: Vec<String>,
    pub facts_snapshot: HashMap<String, Value>,
    pub target_node: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub node_id: String,
    pub execution_result: ExecutionSummary,
    pub updated_facts: HashMap<String, Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub rules_fired: usize,
    pub cycle_count: usize,
    pub execution_time_ms: f64,
    pub success: bool,
}

/// 🎯 Master Node - Coordinates distributed execution
pub struct MasterNode {
    pub node_id: String,
    pub workers: Arc<RwLock<HashMap<String, WorkerNode>>>,
    pub pending_tasks: Arc<Mutex<Vec<DistributedTask>>>,
    pub task_results: Arc<Mutex<HashMap<String, TaskResult>>>,
}

impl MasterNode {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            workers: Arc::new(RwLock::new(HashMap::new())),
            pending_tasks: Arc::new(Mutex::new(Vec::new())),
            task_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a worker node
    pub async fn register_worker(&self, worker: WorkerNode) {
        let mut workers = self.workers.write().await;
        println!("🤝 Master: Registering worker '{}'", worker.node_id);
        workers.insert(worker.node_id.clone(), worker);
    }

    /// Distribute rules across available workers
    pub async fn distribute_execution(
        &self,
        rules: Vec<String>,
        facts: &Facts,
    ) -> Result<Vec<TaskResult>, Box<dyn std::error::Error>> {
        println!("\n🌐 Master: Starting distributed execution");
        println!("   Rules to execute: {}", rules.len());

        let workers = self.workers.read().await;
        if workers.is_empty() {
            return Err("No workers available".into());
        }

        // Simple round-robin distribution
        let worker_count = workers.len();
        let rules_per_worker = (rules.len() + worker_count - 1) / worker_count;
        
        let mut tasks = Vec::new();
        let mut rule_chunks = rules.chunks(rules_per_worker);
        
        for (i, worker) in workers.values().enumerate() {
            if let Some(chunk) = rule_chunks.next() {
                let task_id = format!("task_{}_{}", self.node_id, i);
                
                // Create facts snapshot
                let mut facts_snapshot = HashMap::new();
                // In real implementation, this would be optimized
                for (key, value) in facts.get_all_facts() {
                    facts_snapshot.insert(key, value);
                }

                let task = DistributedTask {
                    task_id: task_id.clone(),
                    rules: chunk.to_vec(),
                    facts_snapshot,
                    target_node: Some(worker.node_id.clone()),
                };

                tasks.push(task);
            }
        }

        println!("   Created {} tasks for {} workers", tasks.len(), worker_count);

        // Execute tasks in parallel
        let mut handles = Vec::new();
        
        for task in tasks {
            let workers_clone = self.workers.clone();
            let handle = tokio::spawn(async move {
                let workers = workers_clone.read().await;
                if let Some(worker) = workers.get(&task.target_node.as_ref().unwrap()) {
                    worker.execute_task(task).await
                } else {
                    Err("Worker not found".into())
                }
            });
            handles.push(handle);
        }

        // Collect results
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => println!("❌ Task execution error: {}", e),
                Err(e) => println!("❌ Task join error: {}", e),
            }
        }

        println!("✅ Master: Collected {} results", results.len());
        Ok(results)
    }

    /// Get cluster status
    pub async fn get_cluster_status(&self) -> Vec<NodeInfo> {
        let workers = self.workers.read().await;
        let mut status = Vec::new();

        // Add master info
        status.push(NodeInfo {
            id: self.node_id.clone(),
            node_type: NodeType::Master,
            status: NodeStatus::Active,
            rules_count: 0,
            facts_count: 0,
        });

        // Add worker info
        for worker in workers.values() {
            status.push(worker.get_info().await);
        }

        status
    }
}

/// 🔨 Worker Node - Executes rules locally
#[derive(Clone)]
pub struct WorkerNode {
    pub node_id: String,
    pub engine: Arc<Mutex<RustRuleEngine>>,
    pub local_facts: Arc<Mutex<Facts>>,
}

impl WorkerNode {
    pub fn new(node_id: String) -> Self {
        let kb = KnowledgeBase::new(&format!("Worker_{}", node_id));
        let config = EngineConfig {
            max_cycles: 5,
            timeout: Some(Duration::from_secs(30)),
            enable_stats: true,
            debug_mode: false,
        };
        let engine = RustRuleEngine::with_config(kb, config);

        Self {
            node_id,
            engine: Arc::new(Mutex::new(engine)),
            local_facts: Arc::new(Mutex::new(Facts::new())),
        }
    }

    /// Execute a distributed task
    pub async fn execute_task(
        &self,
        task: DistributedTask,
    ) -> Result<TaskResult, Box<dyn std::error::Error>> {
        println!("🔨 Worker '{}': Executing task '{}'", self.node_id, task.task_id);
        println!("   Rules to process: {}", task.rules.len());

        let start_time = std::time::Instant::now();

        // Load facts snapshot
        let facts = Facts::new();
        for (key, value) in task.facts_snapshot {
            facts.add_value(&key, value)?;
        }

        // Parse and add rules to engine
        let mut engine = self.engine.lock().await;
        let mut total_rules_fired = 0;
        let mut total_cycles = 0;

        for rule_str in &task.rules {
            // Parse rule from GRL string
            match GRLParser::parse_rules(rule_str) {
                Ok(rules) => {
                    for rule in rules {
                        let _ = engine.knowledge_base.add_rule(rule);
                    }
                }
                Err(e) => {
                    println!("   ⚠️ Failed to parse rule: {}", e);
                    continue;
                }
            }
        }

        // Execute rules
        let result = match engine.execute(&facts) {
            Ok(r) => {
                total_rules_fired += r.rules_fired;
                total_cycles += r.cycle_count;
                println!("   ✅ Execution successful: {} rules fired in {} cycles", 
                    r.rules_fired, r.cycle_count);
                r
            }
            Err(e) => {
                println!("   ❌ Execution failed: {}", e);
                return Ok(TaskResult {
                    task_id: task.task_id,
                    node_id: self.node_id.clone(),
                    execution_result: ExecutionSummary {
                        rules_fired: 0,
                        cycle_count: 0,
                        execution_time_ms: start_time.elapsed().as_secs_f64() * 1000.0,
                        success: false,
                    },
                    updated_facts: HashMap::new(),
                    errors: vec![e.to_string()],
                });
            }
        };

        // Collect updated facts
        let mut updated_facts = HashMap::new();
        for (key, value) in facts.get_all_facts() {
            updated_facts.insert(key, value);
        }

        let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

        Ok(TaskResult {
            task_id: task.task_id,
            node_id: self.node_id.clone(),
            execution_result: ExecutionSummary {
                rules_fired: total_rules_fired,
                cycle_count: total_cycles,
                execution_time_ms: execution_time,
                success: true,
            },
            updated_facts,
            errors: Vec::new(),
        })
    }

    /// Get worker node information
    pub async fn get_info(&self) -> NodeInfo {
        let engine = self.engine.lock().await;
        let facts = self.local_facts.lock().await;

        NodeInfo {
            id: self.node_id.clone(),
            node_type: NodeType::Worker,
            status: NodeStatus::Active,
            rules_count: engine.knowledge_base.rules.len(),
            facts_count: facts.get_all_facts().len(),
        }
    }
}

/// 🎯 Demo Function
pub async fn demo_distributed_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 === Distributed Rule Engine Demo ===");
    println!("This demo simulates distributed rule execution across multiple nodes");
    
    // Create master node
    let master = MasterNode::new("master_001".to_string());
    
    // Create worker nodes
    let worker1 = WorkerNode::new("worker_001".to_string());
    let worker2 = WorkerNode::new("worker_002".to_string());
    let worker3 = WorkerNode::new("worker_003".to_string());
    
    // Register workers with master
    master.register_worker(worker1).await;
    master.register_worker(worker2).await;
    master.register_worker(worker3).await;
    
    // Create test facts
    let facts = Facts::new();
    
    // E-commerce customer data
    let customer = FactHelper::create_customer(
        "Alice Johnson",
        28,
        "alice@example.com",
        "premium", 
        2500.0,
        true
    );
    facts.add_value("Customer", customer)?;
    
    // Transaction data
    let mut transaction_props = HashMap::new();
    transaction_props.insert("Amount".to_string(), Value::Number(750.0));
    transaction_props.insert("Currency".to_string(), Value::String("USD".to_string()));
    transaction_props.insert("Type".to_string(), Value::String("PURCHASE".to_string()));
    transaction_props.insert("RiskScore".to_string(), Value::Number(0.2));
    facts.add_value("Transaction", Value::Object(transaction_props))?;
    
    // Define rules for distributed execution
    let rules = vec![
        // Customer tier rules
        r#"rule "PremiumCustomerBonus" salience 10 {
            when Customer.Tier == "premium" && Customer.SpendingTotal > 2000.0
            then Customer.LoyaltyPoints = Customer.LoyaltyPoints + 500;
        }"#.to_string(),
        
        // Transaction validation rules  
        r#"rule "HighValueTransaction" salience 20 {
            when Transaction.Amount > 500.0 && Transaction.RiskScore < 0.5
            then Transaction.Status = "APPROVED";
        }"#.to_string(),
        
        // Age-based rules
        r#"rule "AdultCustomer" salience 5 {
            when Customer.Age >= 18
            then Customer.IsAdult = true;
        }"#.to_string(),
        
        // Email validation
        r#"rule "ValidEmail" salience 15 {
            when Customer.Email != ""
            then Customer.ContactVerified = true;
        }"#.to_string(),
        
        // Premium benefits
        r#"rule "PremiumBenefits" salience 8 {
            when Customer.Tier == "premium" && Customer.IsAdult == true
            then Customer.HasPremiumBenefits = true;
        }"#.to_string(),
        
        // Transaction processing
        r#"rule "ProcessTransaction" salience 12 {
            when Transaction.Status == "APPROVED" && Customer.ContactVerified == true
            then Transaction.ProcessingStatus = "COMPLETED";
        }"#.to_string(),
    ];
    
    println!("\n📊 Cluster Status:");
    let cluster_status = master.get_cluster_status().await;
    for node in &cluster_status {
        println!("   {} ({}): {:?} - {} rules, {} facts", 
            node.id, 
            match node.node_type { NodeType::Master => "Master", NodeType::Worker => "Worker" },
            node.status,
            node.rules_count,
            node.facts_count
        );
    }
    
    // Execute distributed rules
    println!("\n🚀 Executing rules across distributed nodes...");
    let results = master.distribute_execution(rules, &facts).await?;
    
    // Display results
    println!("\n📈 Execution Results:");
    let mut total_rules_fired = 0;
    let mut total_execution_time = 0.0;
    
    for result in &results {
        println!("   Node '{}': {} rules fired in {:.2}ms", 
            result.node_id,
            result.execution_result.rules_fired,
            result.execution_result.execution_time_ms
        );
        
        total_rules_fired += result.execution_result.rules_fired;
        total_execution_time += result.execution_result.execution_time_ms;
        
        if !result.errors.is_empty() {
            println!("     ❌ Errors: {:?}", result.errors);
        }
    }
    
    println!("\n📊 Summary:");
    println!("   Total rules fired across cluster: {}", total_rules_fired);
    println!("   Average execution time per node: {:.2}ms", total_execution_time / results.len() as f64);
    println!("   Nodes participated: {}", results.len());
    
    // Demonstrate load balancing
    println!("\n⚖️ Load Distribution:");
    for result in &results {
        let load_percentage = (result.execution_result.rules_fired as f64 / total_rules_fired as f64) * 100.0;
        println!("   {}: {:.1}% of total workload", result.node_id, load_percentage);
    }
    
    println!("\n✅ Distributed execution completed successfully!");
    println!("\n🎯 Key Benefits Demonstrated:");
    println!("   🚀 Parallel Execution: Rules executed simultaneously across nodes");
    println!("   ⚖️ Load Balancing: Work distributed evenly among workers");
    println!("   🛡️ Fault Tolerance: Each node operates independently");
    println!("   📊 Monitoring: Centralized result collection and analysis");
    println!("   🌐 Scalability: Easy to add more worker nodes");
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    demo_distributed_execution().await
}
