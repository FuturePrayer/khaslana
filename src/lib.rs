pub mod credentials;
pub mod git;
pub mod proxy;
pub mod storage;
pub mod types;
pub mod workflow;

pub use credentials::{
    CredentialProvider, CredentialRecord, CredentialRequest, CredentialScope, CredentialStore,
    GitCredential, KeyringCredentialStore, MemoryCredentialStore, PromptCredentialProvider,
    RemoteCredentialPolicy, StoredCredentialKind, credential_display_target,
    credential_key_filename, credential_kind_label, credential_record_is_compatible_with_url,
    credential_record_label, credential_record_matches_remote_url, credential_scope_label,
    test_credential_connection,
};
pub use git::{GitService, NoopProgress, ProgressEmitter};
pub use proxy::{CustomProxySettings, NetworkProxyMode, NetworkProxySettings};
pub use storage::{
    AppStorage, DiffEncodingPreferences, LegacyImportSummary, LegacyStoragePaths,
    RemoteCredentialBinding, RemoteCredentialBindings, SessionState, default_database_path,
    default_legacy_storage_paths, legacy_storage_paths,
};
pub use types::*;
pub use workflow::{
    RemoteBranchGuardAction, WorkflowDefinition, WorkflowExecutor, WorkflowInputDefinition,
    WorkflowPreview, WorkflowPreviewStep, WorkflowProgressEvent, WorkflowRunOptions,
    WorkflowRunResult, WorkflowStep, parse_workflow_json5,
};
