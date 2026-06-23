//! Persistencia de cuentas y tokens.
//!
//! - `accounts.json`: lista de cuentas visibles (sin tokens).
//! - `secrets.json`: tokens sensibles por id de cuenta (permisos 0600 en Unix).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config;
use crate::core::paths;
use crate::error::AppResult;
use crate::models::{Account, TokenRecord};

#[derive(Default, Serialize, Deserialize)]
struct AccountsFile {
    #[serde(default)]
    accounts: Vec<Account>,
}

#[derive(Default, Serialize, Deserialize)]
struct SecretsFile {
    #[serde(default)]
    tokens: HashMap<String, TokenRecord>,
}

pub fn load_accounts() -> Vec<Account> {
    config::read_json::<AccountsFile>(&paths::accounts_file())
        .unwrap_or_default()
        .accounts
}

pub fn save_accounts(list: &[Account]) -> AppResult<()> {
    config::write_json_atomic(
        &paths::accounts_file(),
        &AccountsFile { accounts: list.to_vec() },
    )
}

pub fn load_secrets() -> HashMap<String, TokenRecord> {
    config::read_json::<SecretsFile>(&paths::secrets_file())
        .unwrap_or_default()
        .tokens
}

pub fn save_secrets(tokens: &HashMap<String, TokenRecord>) -> AppResult<()> {
    config::write_secret_json(&paths::secrets_file(), &SecretsFile { tokens: tokens.clone() })
}

pub fn get_token(id: &str) -> Option<TokenRecord> {
    load_secrets().get(id).cloned()
}

pub fn set_token(id: &str, rec: TokenRecord) -> AppResult<()> {
    let mut map = load_secrets();
    map.insert(id.to_string(), rec);
    save_secrets(&map)
}

pub fn remove_token(id: &str) -> AppResult<()> {
    let mut map = load_secrets();
    if map.remove(id).is_some() {
        save_secrets(&map)?;
    }
    Ok(())
}
