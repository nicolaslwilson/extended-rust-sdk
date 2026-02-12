pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const USER_AGENT: &str = concat!("ExtendedRustTradingClient/", env!("CARGO_PKG_VERSION"));

pub const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 500;
