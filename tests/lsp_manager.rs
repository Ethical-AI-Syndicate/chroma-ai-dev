use chroma_ai_dev::lsp_manager::{LanguageKind, LspSessionError, LspSessionManager};

#[test]
fn test_language_kind_variants() {
    assert_eq!(LanguageKind::Rust.to_string(), "rust");
    assert_eq!(LanguageKind::TypeScript.to_string(), "typescript");
    assert_eq!(LanguageKind::Python.to_string(), "python");
    assert_eq!(LanguageKind::Go.to_string(), "go");
    assert_eq!(LanguageKind::Java.to_string(), "java");
}

#[test]
fn test_lsp_session_manager_creation() {
    let manager = LspSessionManager::new();
    assert!(manager.list_sessions().is_empty());
}

#[tokio::test]
async fn test_register_lsp_adapter() {
    let mut manager = LspSessionManager::new();

    let result = manager
        .register_adapter(LanguageKind::Rust, "rust-analyzer")
        .await;
    assert!(result.is_ok());

    manager.start_session(LanguageKind::Rust).await.unwrap();

    let sessions = manager.list_sessions();
    assert!(sessions.contains(&LanguageKind::Rust));
}

#[tokio::test]
async fn test_start_stop_session() {
    let mut manager = LspSessionManager::new();

    manager
        .register_adapter(LanguageKind::Rust, "rust-analyzer")
        .await
        .unwrap();

    let start_result = manager.start_session(LanguageKind::Rust).await;
    assert!(start_result.is_ok());

    let status = manager.session_status(LanguageKind::Rust).unwrap();
    assert!(status.is_running());

    let stop_result = manager.stop_session(LanguageKind::Rust).await;
    assert!(stop_result.is_ok());

    let status = manager.session_status(LanguageKind::Rust).unwrap();
    assert!(!status.is_running());
}

#[tokio::test]
async fn test_unregistered_language_error() {
    let mut manager = LspSessionManager::new();

    let result = manager.start_session(LanguageKind::Python).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        LspSessionError::AdapterNotRegistered { .. } => (),
        e => panic!("Expected AdapterNotRegistered error, got {:?}", e),
    }
}
