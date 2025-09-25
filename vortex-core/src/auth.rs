use crate::error::{Result, VortexError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    VmCreate,
    VmRead,
    VmUpdate,
    VmDelete,
    SnapshotCreate,
    SnapshotRestore,
    NetworkManage,
    StorageManage,
    MetricsRead,
    AdminAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub permissions: Vec<Permission>,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self, credentials: &AuthCredentials) -> Result<AuthToken>;
    async fn authorize(&self, token: &str, permission: Permission) -> Result<bool>;
    async fn get_user(&self, user_id: &str) -> Result<Option<User>>;
    async fn refresh_token(&self, token: &str) -> Result<AuthToken>;
}

#[derive(Debug, Clone)]
pub enum AuthCredentials {
    UsernamePassword { username: String, password: String },
    Token { token: String },
    ApiKey { key: String },
}

// No-op auth provider for development
pub struct NoOpAuthProvider;

#[async_trait]
impl AuthProvider for NoOpAuthProvider {
    async fn authenticate(&self, _credentials: &AuthCredentials) -> Result<AuthToken> {
        Ok(AuthToken {
            token: "dev-token".to_string(),
            user_id: "dev-user".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            permissions: vec![Permission::AdminAll],
        })
    }
    
    async fn authorize(&self, _token: &str, _permission: Permission) -> Result<bool> {
        Ok(true) // Allow all in development
    }
    
    async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        Ok(Some(User {
            id: user_id.to_string(),
            username: "developer".to_string(),
            email: Some("dev@vortex.local".to_string()),
            roles: vec!["admin".to_string()],
            permissions: vec![Permission::AdminAll],
        }))
    }
    
    async fn refresh_token(&self, _token: &str) -> Result<AuthToken> {
        self.authenticate(&AuthCredentials::Token { token: "dev-token".to_string() }).await
    }
}

// JWT-based auth provider (stub)
pub struct JwtAuthProvider {
    _secret: String,
    _users: HashMap<String, User>,
}

impl JwtAuthProvider {
    pub fn new(secret: String) -> Self {
        Self {
            _secret: secret,
            _users: HashMap::new(),
        }
    }
}

#[async_trait]
impl AuthProvider for JwtAuthProvider {
    async fn authenticate(&self, _credentials: &AuthCredentials) -> Result<AuthToken> {
        Err(VortexError::AuthError {
            message: "JWT auth provider not yet implemented".to_string(),
        })
    }
    
    async fn authorize(&self, _token: &str, _permission: Permission) -> Result<bool> {
        Err(VortexError::AuthError {
            message: "JWT auth provider not yet implemented".to_string(),
        })
    }
    
    async fn get_user(&self, _user_id: &str) -> Result<Option<User>> {
        Err(VortexError::AuthError {
            message: "JWT auth provider not yet implemented".to_string(),
        })
    }
    
    async fn refresh_token(&self, _token: &str) -> Result<AuthToken> {
        Err(VortexError::AuthError {
            message: "JWT auth provider not yet implemented".to_string(),
        })
    }
}