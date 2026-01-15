# OpenSASE SDK

Rust SDK for connecting to OpenSASE platform and self-hosted services.

## Installation

```toml
[dependencies]
opensase-sdk = { git = "https://github.com/abiolaogu/opensase-sdk" }
```

## Quick Start

```rust
use opensase_sdk::{OpenSaseClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OpenSaseClient::new(Config {
        base_url: "http://localhost:8081".to_string(),
        api_key: "your-api-key".to_string(),
        tenant_id: None,
    })?;

    // CRM
    let contacts = client.crm().list_contacts(None).await?;
    
    // Payments
    let txns = client.payments().list_transactions().await?;
    
    // Support
    let tickets = client.support().list_tickets().await?;

    Ok(())
}
```

## Available Clients

- `client.crm()` - CRM operations
- `client.payments()` - Payment processing
- `client.support()` - Support tickets
- `client.hr()` - HR management
- `client.ecommerce()` - E-commerce
- `client.billing()` - Usage billing

## License

MIT OR Apache-2.0
