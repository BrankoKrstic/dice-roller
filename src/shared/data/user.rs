use leptos::prelude::RwSignal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UserId(i64);

impl UserId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
    pub fn into_inner(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(val: String) -> Self {
        Self(val)
    }
    pub fn into_inner(self) -> String {
        self.0
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Username(String);

impl Username {
    pub fn new(val: String) -> Self {
        Self(val)
    }
    pub fn into_inner(self) -> String {
        self.0
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Password(String);

impl Password {
    pub fn new(val: String) -> Self {
        Self(val)
    }
    pub fn into_inner(self) -> String {
        self.0
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: UserId,
    pub username: Username,
    pub email: Email,
}

#[derive(Clone)]
pub struct AuthContext {
    pub(crate) user: RwSignal<Option<AuthUser>>,
    pub(crate) loading: RwSignal<bool>,
}

impl AuthContext {
    pub fn new(user: Option<AuthUser>) -> Self {
        Self {
            user: RwSignal::new(user),
            loading: RwSignal::new(false),
        }
    }
}
