//! Cryptographic utilities for the Gateway
//!
//! This module provides cryptographic functions for password hashing, API key generation,
//! and authenticated encryption using AES-256-GCM.

#![allow(dead_code)]

pub mod backup;
#[cfg(feature = "gateway")]
pub mod encryption;
pub mod hmac;
pub mod keys;
#[cfg(feature = "gateway")]
pub mod password;
pub mod webhooks;

#[cfg(all(test, feature = "gateway"))]
mod tests;
