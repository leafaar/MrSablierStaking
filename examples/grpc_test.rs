use {
    adrena_abi::{Discriminator, Staking},
    clap::Parser,
    futures::StreamExt,
    std::{collections::HashMap, time::Duration},
    tonic::transport::channel::ClientTlsConfig,
    yellowstone_grpc_client::{GeyserGrpcClient, Interceptor},
    yellowstone_grpc_proto::{
        geyser::{
            SubscribeRequest, SubscribeRequestFilterAccountsFilter,
            SubscribeRequestFilterAccountsFilterMemcmp,
        },
        prelude::{
            subscribe_request_filter_accounts_filter::Filter as AccountsFilterDataOneof,
            subscribe_request_filter_accounts_filter_memcmp::Data as AccountsFilterMemcmpOneof,
            CommitmentLevel, SubscribeRequestFilterAccounts,
        },
    },
};

const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:10000";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
enum ArgsCommitment {
    #[default]
    Processed,
    Confirmed,
    Finalized,
}

impl From<ArgsCommitment> for CommitmentLevel {
    fn from(commitment: ArgsCommitment) -> Self {
        match commitment {
            ArgsCommitment::Processed => CommitmentLevel::Processed,
            ArgsCommitment::Confirmed => CommitmentLevel::Confirmed,
            ArgsCommitment::Finalized => CommitmentLevel::Finalized,
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value_t = String::from(DEFAULT_ENDPOINT))]
    endpoint: String,

    #[clap(long)]
    x_token: Option<String>,

    #[clap(long)]
    commitment: Option<ArgsCommitment>,
}

impl Args {
    fn get_commitment(&self) -> Option<CommitmentLevel> {
        Some(self.commitment.unwrap_or_default().into())
    }

    async fn connect(&self) -> anyhow::Result<GeyserGrpcClient<impl Interceptor>> {
        GeyserGrpcClient::build_from_shared(self.endpoint.clone())?
            .x_token(self.x_token.clone())?
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .tls_config(ClientTlsConfig::new().with_native_roots())?
            .connect()
            .await
            .map_err(Into::into)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Connecting to gRPC endpoint: {}", args.endpoint);

    let mut grpc = args.connect().await?;

    let mut accounts_filter_map = HashMap::new();
    let staking_filter = SubscribeRequestFilterAccountsFilter {
        filter: Some(AccountsFilterDataOneof::Memcmp(
            SubscribeRequestFilterAccountsFilterMemcmp {
                offset: 0,
                data: Some(AccountsFilterMemcmpOneof::Bytes(
                    Staking::DISCRIMINATOR.to_vec(),
                )),
            },
        )),
    };

    accounts_filter_map.insert(
        "staking_test".to_owned(),
        SubscribeRequestFilterAccounts {
            account: vec![],
            owner: vec![adrena_abi::ID.to_string()],
            filters: vec![staking_filter],
            nonempty_txn_signature: None,
        },
    );

    println!("Subscribing to account updates...");

    let (_subscribe_tx, mut stream) = grpc
        .subscribe_with_request(Some(SubscribeRequest {
            accounts: accounts_filter_map,
            commitment: args.get_commitment().map(|c| c.into()),
            ..Default::default()
        }))
        .await?;

    println!("Connected! Waiting for messages...");

    while let Some(message) = stream.next().await {
        match message {
            Ok(msg) => println!("Received message: {:?}", msg),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
}
