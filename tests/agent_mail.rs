use chroma_ai_dev::agent_mail::{AgentMailError, AgentMailer, LeaseMode};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_register_agent() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    let result = mailer
        .lock()
        .await
        .register_agent("agent_1", "Test Agent")
        .await;

    assert!(result.is_ok());
    let registration = result.unwrap();
    assert!(registration.registered);
    assert!(!registration.mailbox_id.is_empty());
}

#[tokio::test]
async fn test_send_and_fetch_message() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    mailer
        .lock()
        .await
        .register_agent("sender", "Sender Agent")
        .await
        .unwrap();
    mailer
        .lock()
        .await
        .register_agent("receiver", "Receiver Agent")
        .await
        .unwrap();

    let msg_id = mailer
        .lock()
        .await
        .send_message("receiver", "thread_1", "Hello world", None)
        .await
        .unwrap();

    assert!(!msg_id.is_empty());

    let inbox = mailer
        .lock()
        .await
        .fetch_inbox("receiver", None, 10)
        .await
        .unwrap();

    assert_eq!(inbox.len(), 1);
    assert_eq!(inbox[0].message, "Hello world");
}

#[tokio::test]
async fn test_ack_message() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    mailer
        .lock()
        .await
        .register_agent("receiver", "Receiver")
        .await
        .unwrap();

    let msg_id = mailer
        .lock()
        .await
        .send_message("receiver", "thread_1", "Test message", None)
        .await
        .unwrap();

    let ack_result = mailer.lock().await.ack_message("receiver", &msg_id).await;

    assert!(ack_result.is_ok());

    let inbox = mailer
        .lock()
        .await
        .fetch_inbox("receiver", None, 10)
        .await
        .unwrap();

    assert!(inbox[0].acknowledged);
}

#[tokio::test]
async fn test_claim_file_lease() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    mailer
        .lock()
        .await
        .register_agent("agent_1", "Agent One")
        .await
        .unwrap();

    let lease_result = mailer
        .lock()
        .await
        .claim_file_lease("agent_1", "src/main.rs", 300, LeaseMode::Read)
        .await;

    assert!(lease_result.is_ok());
    let lease = lease_result.unwrap();
    assert!(!lease.lease_id.is_empty());
    assert_eq!(lease.path, "src/main.rs");
}

#[tokio::test]
async fn test_exclusive_lease_conflict() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    mailer
        .lock()
        .await
        .register_agent("agent_1", "Agent One")
        .await
        .unwrap();
    mailer
        .lock()
        .await
        .register_agent("agent_2", "Agent Two")
        .await
        .unwrap();

    let _lease1 = mailer
        .lock()
        .await
        .claim_file_lease("agent_1", "src/main.rs", 300, LeaseMode::Exclusive)
        .await
        .unwrap();

    let lease2_result = mailer
        .lock()
        .await
        .claim_file_lease("agent_2", "src/main.rs", 300, LeaseMode::Read)
        .await;

    assert!(lease2_result.is_err());
    match lease2_result.unwrap_err() {
        AgentMailError::LeaseConflict { .. } => (),
        e => panic!("Expected LeaseConflict error, got {:?}", e),
    }
}

#[tokio::test]
async fn test_release_file_lease() {
    let mailer = Arc::new(Mutex::new(AgentMailer::new_in_memory().await));

    mailer
        .lock()
        .await
        .register_agent("agent_1", "Agent One")
        .await
        .unwrap();

    let lease = mailer
        .lock()
        .await
        .claim_file_lease("agent_1", "src/main.rs", 300, LeaseMode::Write)
        .await
        .unwrap();

    let release_result = mailer
        .lock()
        .await
        .release_file_lease("agent_1", &lease.lease_id)
        .await;

    assert!(release_result.is_ok());

    let lease2 = mailer
        .lock()
        .await
        .claim_file_lease("agent_1", "src/main.rs", 300, LeaseMode::Write)
        .await;

    assert!(lease2.is_ok());
}
