use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyStrategy {
    FailFast,
    WaitAll,
    Optimistic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureHandling {
    AbortAll,
    ContinueOthers,
    RetryThenAbort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_seconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Pending,
    Ready,
    Running,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: String,
    pub dependencies: Vec<String>,
    pub status: NodeStatus,
}

impl TaskNode {
    pub fn new(id: &str, dependencies: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            dependencies,
            status: NodeStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    nodes: HashMap<String, TaskNode>,
    dependency_strategy: DependencyStrategy,
    failure_handling: FailureHandling,
}

impl TaskGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependency_strategy: DependencyStrategy::FailFast,
            failure_handling: FailureHandling::AbortAll,
        }
    }

    pub fn add_node(&mut self, node: TaskNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn set_dependency_strategy(&mut self, strategy: DependencyStrategy) {
        self.dependency_strategy = strategy;
    }

    pub fn set_failure_handling(&mut self, handling: FailureHandling) {
        self.failure_handling = handling;
    }

    pub fn get_node_status(&self, id: &str) -> Option<NodeStatus> {
        self.nodes.get(id).map(|n| n.status)
    }

    pub fn get_ready_nodes(&self) -> Vec<String> {
        let mut ready = Vec::new();

        for (id, node) in &self.nodes {
            if node.status != NodeStatus::Pending && node.status != NodeStatus::Ready {
                continue;
            }

            let all_deps_completed = node.dependencies.iter().all(|dep_id| {
                self.nodes
                    .get(dep_id)
                    .map(|n| n.status == NodeStatus::Completed)
                    .unwrap_or(false)
            });

            if all_deps_completed {
                ready.push(id.clone());
            }
        }

        ready
    }

    pub fn topological_sort(&self) -> Result<Vec<String>, OrchestratorError> {
        let mut in_degree: HashMap<String, usize> = self
            .nodes
            .iter()
            .map(|(id, node)| {
                let valid_deps = node
                    .dependencies
                    .iter()
                    .filter(|d| self.nodes.contains_key(*d))
                    .count();
                (id.clone(), valid_deps)
            })
            .collect();

        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());

            for (id, node) in &self.nodes {
                if node.dependencies.contains(&node_id) {
                    if let Some(degree) = in_degree.get_mut(id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(id.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(OrchestratorError::CycleDetected);
        }

        Ok(result)
    }

    pub fn mark_completed(&mut self, id: &str) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Completed;
        }
    }

    pub fn mark_failed(&mut self, id: &str, _reason: String) {
        if let Some(node) = self.nodes.get_mut(id) {
            node.status = NodeStatus::Failed;
        }

        if self.failure_handling == FailureHandling::AbortAll {
            for node in self.nodes.values_mut() {
                if node.dependencies.contains(&id.to_string()) && node.status == NodeStatus::Pending
                {
                    node.status = NodeStatus::Blocked;
                }
            }
        }
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error_type", rename_all = "snake_case")]
pub enum OrchestratorError {
    CycleDetected,
    NodeNotFound { id: String },
    InvalidDependency { node: String, dependency: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_node_creation() {
        let node = TaskNode::new("test", vec!["dep1".to_string()]);
        assert_eq!(node.id, "test");
        assert_eq!(node.dependencies, vec!["dep1"]);
        assert_eq!(node.status, NodeStatus::Pending);
    }

    #[test]
    fn test_task_graph_default() {
        let graph = TaskGraph::new();
        assert!(graph.nodes.is_empty());
    }
}
