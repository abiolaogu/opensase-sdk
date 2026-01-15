//! OpenSASE SDK - Connect to OpenSASE platform and self-hosted services

use reqwest::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("HTTP error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    #[error("Config error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, SdkError>;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: String,
    pub api_key: String,
    pub tenant_id: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Config {
            base_url: std::env::var("OPENSASE_API_URL").map_err(|_| SdkError::Config("OPENSASE_API_URL not set".into()))?,
            api_key: std::env::var("OPENSASE_API_KEY").map_err(|_| SdkError::Config("OPENSASE_API_KEY not set".into()))?,
            tenant_id: std::env::var("OPENSASE_TENANT_ID").ok(),
        })
    }
}

#[derive(Clone)]
pub struct OpenSaseClient {
    config: Arc<Config>,
    http: Client,
}

impl OpenSaseClient {
    pub fn new(config: Config) -> Result<Self> {
        let http = Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert("Authorization", format!("Bearer {}", config.api_key).parse().unwrap());
                h.insert("Content-Type", "application/json".parse().unwrap());
                h
            })
            .build()?;
        Ok(Self { config: Arc::new(config), http })
    }

    pub fn from_env() -> Result<Self> { Self::new(Config::from_env()?) }
    
    pub fn crm(&self) -> CrmClient { CrmClient(self.clone()) }
    pub fn payments(&self) -> PaymentsClient { PaymentsClient(self.clone()) }
    pub fn support(&self) -> SupportClient { SupportClient(self.clone()) }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let r = self.http.get(format!("{}{}", self.config.base_url, path)).send().await?;
        self.handle(r).await
    }

    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let r = self.http.post(format!("{}{}", self.config.base_url, path)).json(body).send().await?;
        self.handle(r).await
    }

    async fn handle<T: DeserializeOwned>(&self, r: Response) -> Result<T> {
        if r.status().is_success() { Ok(r.json().await?) }
        else { Err(SdkError::Api { status: r.status().as_u16(), message: r.text().await.unwrap_or_default() }) }
    }
}

#[derive(Debug, Serialize, Deserialize)] pub struct PaginatedResponse<T> { pub data: Vec<T>, pub total: i64, pub page: u32 }

// CRM
#[derive(Clone)] pub struct CrmClient(OpenSaseClient);
#[derive(Debug, Serialize, Deserialize)] pub struct Contact { pub id: uuid::Uuid, pub email: String, pub first_name: Option<String>, pub last_name: Option<String> }

impl CrmClient {
    pub async fn list_contacts(&self) -> Result<Vec<Contact>> { let r: PaginatedResponse<Contact> = self.0.get("/api/v1/contacts").await?; Ok(r.data) }
    pub async fn get_contact(&self, id: uuid::Uuid) -> Result<Contact> { self.0.get(&format!("/api/v1/contacts/{}", id)).await }
}

// Payments
#[derive(Clone)] pub struct PaymentsClient(OpenSaseClient);
#[derive(Debug, Serialize, Deserialize)] pub struct Transaction { pub id: uuid::Uuid, pub reference: String, pub amount: i64, pub status: String }

impl PaymentsClient {
    pub async fn list_transactions(&self) -> Result<Vec<Transaction>> { let r: PaginatedResponse<Transaction> = self.0.get("/api/v1/transactions").await?; Ok(r.data) }
}

// Support
#[derive(Clone)] pub struct SupportClient(OpenSaseClient);
#[derive(Debug, Serialize, Deserialize)] pub struct Ticket { pub id: uuid::Uuid, pub ticket_number: String, pub subject: String, pub status: String }

impl SupportClient {
    pub async fn list_tickets(&self) -> Result<Vec<Ticket>> { let r: PaginatedResponse<Ticket> = self.0.get("/api/v1/tickets").await?; Ok(r.data) }
}
