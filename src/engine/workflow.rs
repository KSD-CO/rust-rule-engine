use crate::errors::Result;
use crate::types::Value;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents the status of a workflow
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStatus {
    /// Workflow is currently running
    Running,
    /// Workflow has completed successfully
    Completed,
    /// Workflow has failed
    Failed,
    /// Workflow is paused
    Paused,
    /// Workflow is waiting for a scheduled task
    Waiting,
}

/// Represents a scheduled task in the workflow
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Name of the rule to execute
    pub rule_name: String,
    /// When to execute the task
    pub execute_at: Instant,
    /// Associated workflow ID
    pub workflow_id: Option<String>,
}

/// Tracks the state of a workflow execution
#[derive(Debug, Clone)]
pub struct WorkflowState {
    /// Unique workflow identifier
    pub workflow_id: String,
    /// Current active step/agenda group
    pub current_step: Option<String>,
    /// List of completed steps
    pub completed_steps: Vec<String>,
    /// Workflow-specific data storage
    pub workflow_data: HashMap<String, Value>,
    /// Current status of the workflow
    pub status: WorkflowStatus,
    /// Workflow start time
    pub started_at: Instant,
    /// Workflow completion time
    pub completed_at: Option<Instant>,
}

impl WorkflowState {
    /// Create a new workflow state
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            current_step: None,
            completed_steps: Vec::new(),
            workflow_data: HashMap::new(),
            status: WorkflowStatus::Running,
            started_at: Instant::now(),
            completed_at: None,
        }
    }

    /// Mark a step as completed
    pub fn complete_step(&mut self, step: String) {
        if let Some(current) = &self.current_step {
            if current == &step {
                self.completed_steps.push(step);
                self.current_step = None;
            }
        }
    }

    /// Set the current active step
    pub fn set_current_step(&mut self, step: String) {
        self.current_step = Some(step);
    }

    /// Complete the workflow
    pub fn complete(&mut self) {
        self.status = WorkflowStatus::Completed;
        self.completed_at = Some(Instant::now());
        self.current_step = None;
    }

    /// Fail the workflow
    pub fn fail(&mut self) {
        self.status = WorkflowStatus::Failed;
        self.completed_at = Some(Instant::now());
        self.current_step = None;
    }

    /// Set workflow data
    pub fn set_data(&mut self, key: String, value: Value) {
        self.workflow_data.insert(key, value);
    }

    /// Get workflow data
    pub fn get_data(&self, key: &str) -> Option<&Value> {
        self.workflow_data.get(key)
    }

    /// Get workflow duration
    pub fn duration(&self) -> Duration {
        match self.completed_at {
            Some(end) => end.duration_since(self.started_at),
            None => Instant::now().duration_since(self.started_at),
        }
    }
}

/// Manages workflow execution and scheduling
#[derive(Debug)]
pub struct WorkflowEngine {
    /// Active workflows by ID
    workflows: HashMap<String, WorkflowState>,
    /// Scheduled tasks queue
    scheduled_tasks: Vec<ScheduledTask>,
    /// Queue of agenda groups to activate
    agenda_activation_queue: Vec<String>,
    /// Workflow execution counter
    workflow_counter: u64,
}

impl WorkflowEngine {
    /// Create a new workflow engine
    pub fn new() -> Self {
        Self {
            workflows: HashMap::new(),
            scheduled_tasks: Vec::new(),
            agenda_activation_queue: Vec::new(),
            workflow_counter: 0,
        }
    }

    /// Start a new workflow
    pub fn start_workflow(&mut self, workflow_name: Option<String>) -> String {
        self.workflow_counter += 1;
        let workflow_id =
            workflow_name.unwrap_or_else(|| format!("workflow_{}", self.workflow_counter));

        let workflow_state = WorkflowState::new(workflow_id.clone());
        self.workflows.insert(workflow_id.clone(), workflow_state);

        println!("🔄 Started workflow: {}", workflow_id);
        workflow_id
    }

    /// Activate an agenda group for workflow progression
    pub fn activate_agenda_group(&mut self, group: String) {
        self.agenda_activation_queue.push(group.clone());
        println!("🎯 Queued agenda group activation: {}", group);
    }

    /// Schedule a rule to execute after a delay
    pub fn schedule_rule(&mut self, rule_name: String, delay_ms: u64, workflow_id: Option<String>) {
        let task = ScheduledTask {
            rule_name: rule_name.clone(),
            execute_at: Instant::now() + Duration::from_millis(delay_ms),
            workflow_id,
        };

        self.scheduled_tasks.push(task);
        println!(
            "⏰ Scheduled rule '{}' to execute in {}ms",
            rule_name, delay_ms
        );
    }

    /// Complete a workflow
    pub fn complete_workflow(&mut self, workflow_name: String) {
        if let Some(workflow) = self.workflows.get_mut(&workflow_name) {
            workflow.complete();
            println!("✅ Completed workflow: {}", workflow_name);
        }
    }

    /// Set workflow data
    pub fn set_workflow_data(&mut self, workflow_id: &str, key: String, value: Value) {
        if let Some(workflow) = self.workflows.get_mut(workflow_id) {
            workflow.set_data(key.clone(), value);
            println!(
                "💾 Set workflow data: {} = {:?}",
                key,
                workflow.get_data(&key)
            );
        }
    }

    /// Get the next agenda group to activate
    pub fn get_next_agenda_group(&mut self) -> Option<String> {
        if !self.agenda_activation_queue.is_empty() {
            Some(self.agenda_activation_queue.remove(0))
        } else {
            None
        }
    }

    /// Get ready scheduled tasks
    pub fn get_ready_tasks(&mut self) -> Vec<ScheduledTask> {
        let now = Instant::now();
        let mut ready_tasks = Vec::new();

        self.scheduled_tasks.retain(|task| {
            if task.execute_at <= now {
                ready_tasks.push(task.clone());
                false // Remove from queue
            } else {
                true // Keep in queue
            }
        });

        if !ready_tasks.is_empty() {
            println!(
                "⚡ {} scheduled tasks are ready for execution",
                ready_tasks.len()
            );
        }

        ready_tasks
    }

    /// Get the next pending agenda activation (for syncing with agenda manager)
    pub fn get_next_pending_agenda_activation(&mut self) -> Option<String> {
        if !self.agenda_activation_queue.is_empty() {
            Some(self.agenda_activation_queue.remove(0))
        } else {
            None
        }
    }

    /// Get workflow state by ID
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&WorkflowState> {
        self.workflows.get(workflow_id)
    }

    /// Get all active workflows
    pub fn get_active_workflows(&self) -> Vec<&WorkflowState> {
        self.workflows
            .values()
            .filter(|w| w.status == WorkflowStatus::Running || w.status == WorkflowStatus::Waiting)
            .collect()
    }

    /// Get workflow statistics
    pub fn get_workflow_stats(&self) -> WorkflowStats {
        let total = self.workflows.len();
        let running = self
            .workflows
            .values()
            .filter(|w| w.status == WorkflowStatus::Running)
            .count();
        let completed = self
            .workflows
            .values()
            .filter(|w| w.status == WorkflowStatus::Completed)
            .count();
        let failed = self
            .workflows
            .values()
            .filter(|w| w.status == WorkflowStatus::Failed)
            .count();
        let scheduled_tasks = self.scheduled_tasks.len();

        WorkflowStats {
            total_workflows: total,
            running_workflows: running,
            completed_workflows: completed,
            failed_workflows: failed,
            pending_scheduled_tasks: scheduled_tasks,
            pending_agenda_activations: self.agenda_activation_queue.len(),
        }
    }

    /// Clean up completed workflows older than specified duration
    pub fn cleanup_completed_workflows(&mut self, older_than: Duration) {
        let cutoff = Instant::now() - older_than;
        let initial_count = self.workflows.len();

        self.workflows.retain(|_, workflow| {
            if workflow.status == WorkflowStatus::Completed
                || workflow.status == WorkflowStatus::Failed
            {
                if let Some(completed_at) = workflow.completed_at {
                    completed_at > cutoff
                } else {
                    true // Keep if no completion time
                }
            } else {
                true // Keep active workflows
            }
        });

        let cleaned = initial_count - self.workflows.len();
        if cleaned > 0 {
            println!("🧹 Cleaned up {} completed workflows", cleaned);
        }
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow execution statistics
#[derive(Debug, Clone)]
pub struct WorkflowStats {
    /// Total number of workflows created
    pub total_workflows: usize,
    /// Number of currently running workflows
    pub running_workflows: usize,
    /// Number of completed workflows
    pub completed_workflows: usize,
    /// Number of failed workflows
    pub failed_workflows: usize,
    /// Number of pending scheduled tasks
    pub pending_scheduled_tasks: usize,
    /// Number of pending agenda group activations
    pub pending_agenda_activations: usize,
}

/// Workflow execution result
#[derive(Debug, Clone)]
pub struct WorkflowResult {
    /// Workflow execution was successful
    pub success: bool,
    /// Number of workflow steps executed
    pub steps_executed: usize,
    /// Total execution time
    pub execution_time: Duration,
    /// Final workflow status
    pub final_status: WorkflowStatus,
    /// Any error message if failed
    pub error_message: Option<String>,
}

impl WorkflowResult {
    /// Create a successful workflow result
    pub fn success(steps_executed: usize, execution_time: Duration) -> Self {
        Self {
            success: true,
            steps_executed,
            execution_time,
            final_status: WorkflowStatus::Completed,
            error_message: None,
        }
    }

    /// Create a failed workflow result
    pub fn failure(error_message: String) -> Self {
        Self {
            success: false,
            steps_executed: 0,
            execution_time: Duration::from_millis(0),
            final_status: WorkflowStatus::Failed,
            error_message: Some(error_message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_state_creation() {
        let workflow = WorkflowState::new("test_workflow".to_string());
        assert_eq!(workflow.workflow_id, "test_workflow");
        assert_eq!(workflow.status, WorkflowStatus::Running);
        assert!(workflow.current_step.is_none());
        assert!(workflow.completed_steps.is_empty());
    }

    #[test]
    fn test_workflow_engine_creation() {
        let engine = WorkflowEngine::new();
        assert_eq!(engine.workflows.len(), 0);
        assert_eq!(engine.scheduled_tasks.len(), 0);
    }

    #[test]
    fn test_start_workflow() {
        let mut engine = WorkflowEngine::new();
        let workflow_id = engine.start_workflow(Some("test".to_string()));
        assert_eq!(workflow_id, "test");
        assert!(engine.get_workflow("test").is_some());
    }

    #[test]
    fn test_schedule_rule() {
        let mut engine = WorkflowEngine::new();
        engine.schedule_rule("test_rule".to_string(), 1000, None);
        assert_eq!(engine.scheduled_tasks.len(), 1);
    }

    #[test]
    fn test_workflow_stats() {
        let mut engine = WorkflowEngine::new();
        engine.start_workflow(Some("test1".to_string()));
        engine.start_workflow(Some("test2".to_string()));

        let stats = engine.get_workflow_stats();
        assert_eq!(stats.total_workflows, 2);
        assert_eq!(stats.running_workflows, 2);
    }
}
