use phoenix_sdk::sdk_client::{MarketEventDetails, PhoenixEvent, SDKClient};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::GetConfirmedSignaturesForAddress2Config;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair};
use std::str::FromStr;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use solana_sdk::signature::Signature;
use tokio::sync::mpsc;
use tokio_stream::Stream;

const SOLANA_RPC: &str = "https://api.mainnet-beta.solana.com";
const SOL_MARKET: &str = "4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg";

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrderPlaced {
    price_in_ticks: u64,
    base_lots_placed: u64,
}

pub struct EventFetcher {
    sdk_client: Arc<SDKClient>,
    sol_market: Pubkey,
}

impl EventFetcher {
    pub async fn new() -> anyhow::Result<Self> {
        let keypair = Keypair::new();
        let rpc_client =
            RpcClient::new_with_commitment(SOLANA_RPC.to_string(), CommitmentConfig::confirmed());
        let ellipsis_client =
            ellipsis_client::ellipsis_client::EllipsisClient::from_rpc(rpc_client, &keypair)?;
        let sdk_client = SDKClient::new_from_ellipsis_client(ellipsis_client).await?;

        let sol_market = Pubkey::from_str(SOL_MARKET).unwrap();

        Ok(Self {
            sdk_client: Arc::new(sdk_client),
            sol_market,
        })
    }

    pub fn run(&self) -> impl Stream<Item = NewOrderPlaced> + '_ {
        let (tx, rx) = mpsc::channel(10);
        let sdk_client = self.sdk_client.clone();
        let sol_market = self.sol_market;

        tokio::spawn(async move {
            let mut until: Option<Signature> = None;
            loop {
                dbg!("Ping");
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
                        tokio::spawn(
                            async move { sdk.parse_events_from_transaction(&signature).await },
                        );
                    handles.push(handle);
                }

                for handle in handles {
                    let events = handle.await.unwrap();
                    if let Some(events) = events {
                        for event in events.iter() {
                            let ev: PhoenixEvent = *event;
                            let market_details = ev.details;

                            match market_details {
                                MarketEventDetails::Place(e2) => {
                                    let basic_place = NewOrderPlaced {
                                        price_in_ticks: e2.price_in_ticks,
                                        base_lots_placed: e2.base_lots_placed,
                                    };

                                    if let Err(e) = tx.send(basic_place).await {
                                        eprintln!("Error sending event through channel: {:?}", e);
                                    }
                                }
                                _ => {
                                    
                                }
                            }
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}
