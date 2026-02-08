use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CredentialStoreEncryptionMode {
    None,
    Keyed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderAuthMethod {
    ApiKey,
    OauthToken,
    Adc,
    SessionToken,
}

impl ProviderAuthMethod {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            ProviderAuthMethod::ApiKey => "api_key",
            ProviderAuthMethod::OauthToken => "oauth_token",
            ProviderAuthMethod::Adc => "adc",
            ProviderAuthMethod::SessionToken => "session_token",
        }
    }
}
