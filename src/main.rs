use phoenix_sdk::sdk_client::MarketEventDetails;
use phoenix_sdk::sdk_client::PhoenixEvent;
use phoenix_sdk::sdk_client::SDKClient;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::signature::Signature;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use std::sync::Arc;

const SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";
const SOL_MARKET: &str = "4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg";

#[derive(Debug, Serialize, Deserialize)]
struct NewOrderPlaced {
    price_in_ticks: u64,
    base_lots_placed: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");

    let keypair = Keypair::new();

    let a = ellipsis_client::ellipsis_client::EllipsisClient::from_rpc(
        RpcClient::new_with_commitment(SOLANA_RPC.to_string(), CommitmentConfig::confirmed()),
        &keypair,
    )?;

    let sdk_client = Arc::new(SDKClient::new_from_ellipsis_client(a).await?);

    let sol_market = Pubkey::from_str(SOL_MARKET).unwrap();

    let mut until = None;

    let mut vec_events: Vec<String> = vec![];

    loop {
        dbg!("a");
        let config = match until {
            None => GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: None,
                limit: Some(1),
                commitment: Some(CommitmentConfig::confirmed()),
            },
            Some(until) => GetConfirmedSignaturesForAddress2Config {
                before: None,
                until: Some(until),
                limit: None,
                commitment: Some(CommitmentConfig::confirmed()),
            },
        };
        let signatures = sdk_client
            .client
            .get_signatures_for_address_with_config(&sol_market, config)
            .await
            .unwrap_or_default()
            .iter()
            .map(|tx| Signature::from_str(&tx.signature).unwrap())
            .rev()
            .collect::<Vec<_>>();
        if !signatures.is_empty() {
            until = Some(signatures[0]);
        }
        let mut handles = vec![];
        for signature in signatures {
            let sdk = sdk_client.clone();
            let handle =
                tokio::spawn(async move { sdk.parse_events_from_transaction(&signature).await });
            handles.push(handle);
        }

        for handle in handles {
            let events = handle.await?;
            events.map(|events| {
                events.iter().for_each(|e| {
                    let ev: PhoenixEvent = e.clone();
                    let market_details = ev.details;

                    match market_details {
                        MarketEventDetails::Place(e2) => {
                            let basic_place = NewOrderPlaced {
                                price_in_ticks: e2.price_in_ticks,
                                base_lots_placed: e2.base_lots_placed,
                            };
                            dbg!(e2);
                        }
                        _ => {}
                    }
                });
            });
        }

        // Note: this is a basic polling loop, if there are >1000 signatures in 200ms
        // events will get dropped
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    Ok(())
}
