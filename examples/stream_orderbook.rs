use extended_rust_sdk::{OrderBook, PerpetualStreamClient, TESTNET_CONFIG};
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a stream client
    let stream_client = PerpetualStreamClient::new(&TESTNET_CONFIG.stream_url);

    // 2. Subscribe to orderbook updates for BTC-USD
    let mut conn = stream_client
        .subscribe_to_orderbooks(Some("BTC-USD"), Some(10))
        .await?;

    // 3. Maintain a local orderbook
    let mut orderbook = OrderBook::new();
    let mut is_first = true;

    println!("Listening for BTC-USD orderbook updates...");

    while let Some(msg) = conn.next().await {
        let update = msg?;
        if let Some(data) = &update.data {
            if is_first {
                orderbook.init_orderbook(data);
                is_first = false;
                println!("Orderbook initialized with snapshot");
            } else {
                orderbook.update_orderbook(data);
            }

            if let (Some(bid), Some(ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
                println!(
                    "Best bid: {} @ {} | Best ask: {} @ {} | Spread: {:?}",
                    bid.amount,
                    bid.price,
                    ask.amount,
                    ask.price,
                    orderbook.spread()
                );
            }
        }
    }

    Ok(())
}
