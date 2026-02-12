use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct StarknetDomain {
    pub name: String,
    pub version: String,
    pub chain_id: String,
    pub revision: String,
}

#[derive(Debug, Clone)]
pub struct EndpointConfig {
    pub chain_rpc_url: String,
    pub api_base_url: String,
    pub stream_url: String,
    pub onboarding_url: String,
    pub signing_domain: String,
    pub collateral_asset_contract: String,
    pub asset_operations_contract: String,
    pub collateral_asset_on_chain_id: String,
    pub collateral_decimals: u32,
    pub collateral_asset_id: String,
    pub starknet_domain: StarknetDomain,
}

pub static TESTNET_CONFIG: LazyLock<EndpointConfig> = LazyLock::new(|| EndpointConfig {
    chain_rpc_url: "https://rpc.sepolia.org".into(),
    api_base_url: "https://api.starknet.sepolia.extended.exchange/api/v1".into(),
    stream_url: "wss://api.starknet.sepolia.extended.exchange/stream.extended.exchange/v1".into(),
    onboarding_url: "https://api.starknet.sepolia.extended.exchange".into(),
    signing_domain: "starknet.sepolia.extended.exchange".into(),
    collateral_asset_contract:
        "0x31857064564ed0ff978e687456963cba09c2c6985d8f9300a1de4962fafa054".into(),
    asset_operations_contract: String::new(),
    collateral_asset_on_chain_id: "0x1".into(),
    collateral_decimals: 6,
    collateral_asset_id: "0x1".into(),
    starknet_domain: StarknetDomain {
        name: "Perpetuals".into(),
        version: "v0".into(),
        chain_id: "SN_SEPOLIA".into(),
        revision: "1".into(),
    },
});

pub static MAINNET_CONFIG: LazyLock<EndpointConfig> = LazyLock::new(|| EndpointConfig {
    chain_rpc_url: String::new(),
    api_base_url: "https://api.starknet.extended.exchange/api/v1".into(),
    stream_url: "wss://api.starknet.extended.exchange/stream.extended.exchange/v1".into(),
    onboarding_url: "https://api.starknet.extended.exchange".into(),
    signing_domain: "extended.exchange".into(),
    collateral_asset_contract: String::new(),
    asset_operations_contract: String::new(),
    collateral_asset_on_chain_id: "0x1".into(),
    collateral_decimals: 6,
    collateral_asset_id: "0x1".into(),
    starknet_domain: StarknetDomain {
        name: "Perpetuals".into(),
        version: "v0".into(),
        chain_id: "SN_MAIN".into(),
        revision: "1".into(),
    },
});
