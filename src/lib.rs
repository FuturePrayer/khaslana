pub mod credentials;
pub mod git;
pub mod types;

pub use credentials::{
    CredentialProvider, CredentialRecord, CredentialRequest, CredentialScope, CredentialStore,
    GitCredential, KeyringCredentialStore, MemoryCredentialStore, PromptCredentialProvider,
    StoredCredentialKind, credential_display_target, credential_key_filename,
    credential_kind_label, credential_record_label, credential_scope_label,
    test_credential_connection,
};
pub use git::{GitService, NoopProgress, ProgressEmitter};
pub use types::*;
