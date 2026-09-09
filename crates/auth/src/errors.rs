use thiserror::Error;

/// Authentication errors.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("2FA code required")]
    TwoFactorRequired,

    #[error("Invalid 2FA code")]
    Invalid2FACode,

    #[error("Account banned: {0}")]
    AccountBanned(String),

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid response from server: {0}")]
    InvalidResponse(String),

    #[error("HTTP {status} from the auth server: {body}")]
    HttpStatus { status: u16, body: String },

    #[error("Missing '{field}' in the auth server response")]
    MissingField { field: &'static str },

    #[error("This Microsoft account doesn't own Minecraft")]
    MinecraftNotOwned,

    #[error("Xbox Live is not available in this account's country")]
    XboxLiveUnavailable,

    #[error("This provider does not support token verification")]
    VerificationUnsupported,

    #[error("Username must be between {min} and {max} characters")]
    UsernameLength { min: usize, max: usize },

    #[error("Username can only contain letters, numbers and underscores")]
    UsernameCharset,

    #[error("Token expired or invalid")]
    InvalidToken,

    #[error("User cancelled authentication")]
    Cancelled,

    #[error("Device code expired")]
    DeviceCodeExpired,

    #[error("Authentication timeout")]
    Timeout,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(feature = "keyring")]
    #[error("OS keychain error: {0}")]
    Keyring(#[from] ::keyring::Error),

    #[error("{0}")]
    Custom(String),
}
