pub mod amakna;

use std::time::Instant;
use std::collections::hash_map::HashMap;
use reqwest::RedirectPolicy;

/// Metadata of running engine which contains all details
pub struct EngineSession {
    account: Account,
    started_at: Instant,
    http_client: reqwest::Client,
}

impl EngineSession {
    /// Create a new engine session
    pub fn new(account: Account) -> Result<Self, String> {
        println!("Creating a new Amakna EngineExecutor");

        let client = reqwest::Client::builder()
            .redirect(RedirectPolicy::none())
            .build();
        match client {
            Ok(client) => {
                println!("Client created.");
                Ok(EngineSession {
                    account,
                    started_at: Instant::now(),
                    http_client: client,
                })
            },
            Err(err) => {
                eprintln!("Impossible to init the HTTP client");
                Err("Impossible to init the HTTP client".to_string())
            }
        }
    }

    pub fn get_account(&self) -> &Account {
        &self.account
    }

    pub fn get_started_at(&self) -> &Instant {
        &self.started_at
    }

    pub fn get_http_client(&self) -> &reqwest::Client {
        &self.http_client
    }
}

/// Minimal behaviours of BotEngine
pub trait EngineExecutor where {
    fn run(&mut self) -> Result<bool, String>;
    fn get_website() -> Website;
}

#[derive(Debug)]
pub struct Account {
    email: String,
    password: String,
}

///
/// Account's payload
///
impl Account {

    pub fn new(email: String, password: String) -> Self {
        Self {
            email,
            password,
        }
    }

    /// Convert login request payload to hash map
    pub fn get_param_map(&self) -> HashMap<String, String> {
        let mut hash_map: HashMap<String, String> = HashMap::new();
        hash_map.insert("user_name".to_string(), self.email.clone());
        hash_map.insert("user_password".to_string(), self.password.clone());

        hash_map
    }
}


///
/// Current website meta-data
pub struct Website {
    name: String,
    host: String,
    login_endpoint_path: String,
    vote_endpoint_path: String,
    index_endpoint_path: String,
}

/// Different captcha which existing to protect actions
pub enum Captcha {
    RE_CAPTCHA,
    RE_CAPTCHA_V2,
    NONE,
}

/// Represent protections for different main actions
pub struct ActionsProtection {
    login: Captcha,
    vote: Captcha,
}

/// By default all main actions have no protections (captcha)
impl Default for ActionsProtection {
    fn default() -> Self {
        println!("Creating a default Action protection");
        ActionsProtection {
            login: Captcha::NONE,
            vote: Captcha::NONE,
        }
    }
}