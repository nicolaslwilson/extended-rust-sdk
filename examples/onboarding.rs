use extended_rust_sdk::{UserClient, TESTNET_CONFIG};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Your Ethereum private key (DO NOT hardcode in production!)
    let eth_private_key = "0x_YOUR_ETH_PRIVATE_KEY".to_string();

    // 2. Create a user client with an L1 key provider
    let pk = eth_private_key.clone();
    let user_client = UserClient::new(
        TESTNET_CONFIG.clone(),
        move || pk.clone(),
    )?;

    // 3. Onboard (register) the account
    //    This derives L2 keys from L1, signs EIP-712 payloads, and registers on the exchange.
    let onboarded = user_client.onboard(None).await?;
    println!("Onboarded! Account: {:?}", onboarded.account);
    println!("L2 public key: {}", onboarded.l2_key_pair.public_hex());

    // 4. List all accounts
    let accounts = user_client.get_accounts().await?;
    for acct in &accounts {
        println!(
            "Account #{}: {} (vault: {})",
            acct.account.account_index,
            acct.account.description,
            acct.account.l2_vault,
        );
    }

    // 5. Create an API key for trading
    if let Some(acct) = accounts.first() {
        let api_key = user_client
            .create_account_api_key(&acct.account, Some("My Trading Bot"))
            .await?;
        println!("API key created: {}", api_key);
    }

    Ok(())
}
