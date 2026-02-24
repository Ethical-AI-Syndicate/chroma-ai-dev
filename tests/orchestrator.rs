use chroma_ai_dev::orchestrator::{
    DependencyStrategy, FailureHandling, NodeStatus, RetryPolicy, TaskGraph, TaskNode,
};

#[test]
fn test_dag_dependency_ordering() {
    let mut graph = TaskGraph::new();

    let node_a = TaskNode::new("a", vec![]);
    let node_b = TaskNode::new("b", vec!["a".to_string()]);
    let node_c = TaskNode::new("c", vec!["b".to_string()]);

    graph.add_node(node_a);
    graph.add_node(node_b);
    graph.add_node(node_c);

    let execution_order = graph.topological_sort().unwrap();

    assert_eq!(execution_order, vec!["a", "b", "c"]);
}

#[test]
fn test_parallel_independent_nodes() {
    let mut graph = TaskGraph::new();

    let node_a = TaskNode::new("a", vec![]);
    let node_b = TaskNode::new("b", vec![]);
    let node_c = TaskNode::new("c", vec![]);

    graph.add_node(node_a);
    graph.add_node(node_b);
    graph.add_node(node_c);

    let ready = graph.get_ready_nodes();

    assert_eq!(ready.len(), 3);
    assert!(ready.contains(&"a".to_string()));
    assert!(ready.contains(&"b".to_string()));
    assert!(ready.contains(&"c".to_string()));
}

#[test]
fn test_join_waits_for_prerequisites() {
    let mut graph = TaskGraph::new();

    let node_a = TaskNode::new("a", vec![]);
    let node_b = TaskNode::new("b", vec![]);
    let node_c = TaskNode::new("c", vec!["a".to_string(), "b".to_string()]);

    graph.add_node(node_a.clone());
    graph.add_node(node_b.clone());
    graph.add_node(node_c);

    assert_eq!(graph.get_ready_nodes().len(), 2);

    graph.mark_completed("a");
    assert_eq!(graph.get_ready_nodes().len(), 1);

    graph.mark_completed("b");
    let ready = graph.get_ready_nodes();
    assert_eq!(ready.len(), 1);
    assert!(ready.contains(&"c".to_string()));
}

#[test]
fn test_failure_handling_abort_all() {
    let mut graph = TaskGraph::new();
    graph.set_failure_handling(FailureHandling::AbortAll);

    let node_a = TaskNode::new("a", vec![]);
    let node_b = TaskNode::new("b", vec!["a".to_string()]);

    graph.add_node(node_a);
    graph.add_node(node_b);

    graph.mark_completed("a");
    graph.mark_failed("a", "simulated failure".to_string());

    let status = graph.get_node_status("b").unwrap();
    assert_eq!(status, NodeStatus::Blocked);
}

#[test]
fn test_retry_policy_configuration() {
    let policy = RetryPolicy {
        max_retries: 3,
        backoff_seconds: 5,
    };

    assert_eq!(policy.max_retries, 3);
    assert_eq!(policy.backoff_seconds, 5);
}

#[test]
fn test_dependency_strategy_fail_fast() {
    let strategy = DependencyStrategy::FailFast;
    assert!(matches!(strategy, DependencyStrategy::FailFast));
}
