use std::{f32::consts::E, fmt::Display};

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use leptos::html::P;
use regex::Regex;
use serde::{
    de::{self, Error, Visitor},
    Deserialize, Deserializer,
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserName(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHashed(String);

#[derive(Debug, Error)]
#[error("Password error {0}")]
struct PasswordError(String);

#[derive(Debug, Error)]
enum DeserializeError {
    #[error("Deserialize Error: {0}")]
    Message(String),
}

impl de::Error for DeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        DeserializeError::Message(msg.to_string())
    }
}

impl PasswordHashed {
    pub fn from_unhashed(password: &str) -> Result<Self, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| Self(hash.to_string()))
            .map_err(|error| PasswordError(error.to_string()))
    }
    pub fn new(password: String) -> Self {
        Self(password)
    }
}

fn deserialize_password<'de, D>(deserializer: D) -> Result<PasswordHashed, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    PasswordHashed::from_unhashed(s).map_err(de::Error::custom)
}

fn deserialize_username<'de, D>(deserializer: D) -> Result<UserName, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;
    let s = s.trim();

    if s.len() < 2 || s.len() > 64 {
        Err(de::Error::invalid_length(
            s.len(),
            &"A string between 2 and 64 characters long",
        ))
    } else {
        Ok(UserName(s.to_string()))
    }
}

fn deserialize_email<'de, D>(deserializer: D) -> Result<Email, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = de::Deserialize::deserialize(deserializer)?;

    let s = s.trim();

    let rx = Regex::new(
        r#"(?:[a-z0-9!#$%&'*+\x2f=?^_`\x7b-\x7d~\x2d]+(?:\.[a-z0-9!#$%&'*+\x2f=?^_`\x7b-\x7d~\x2d]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9\x2d]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9\x2d]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9\x2d]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#,
    ).map_err(de::Error::custom)?;

    rx.captures(s)
        .ok_or(de::Error::custom("Invalid password"))
        .map(|_| Email(s.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(deserialize_with = "deserialize_email")]
    email: Email,
    #[serde(deserialize_with = "deserialize_username")]
    user_name: UserName,
    #[serde(deserialize_with = "deserialize_password")]
    password: PasswordHashed,
}

impl User {
    pub fn new(email: String, user_name: String, password: String) -> Self {
        Self {
            email: Email(email),
            user_name: UserName(user_name),
            password: PasswordHashed::new(password),
        }
    }
}
