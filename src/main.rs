// use clap::{Arg, Command};
// use futures::{sink::SinkExt, stream::StreamExt};
// use serde::Deserialize;
// use log::info;
// use std::error::Error;
// use std::fs;
// use tokio::time::{interval, Duration};
// use tonic::transport::channel::ClientTlsConfig;
// use yellowstone_grpc_client::GeyserGrpcClient;
// use yellowstone_grpc_proto::prelude::{
//     subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots,
//     SubscribeRequestPing, SubscribeUpdatePong, SubscribeUpdateSlot,
// };

// #[derive(Debug, Deserialize)]
// struct Config {
//     solana_rpc_url: String,
//     sender_privkey: String,
//     recipient_pubkey: String,
//     amount: u64,
// }

// fn load_config(config_path: &str) -> Result<Config, Box<dyn Error>> {
//     let config_content = fs::read_to_string(config_path)?;
//     let config: Config = serde_yaml::from_str(&config_content)?;
//     Ok(config)
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     let matches = Command::new("Solana Geyser CLI")
//         .version("1.0")
//         .author("Разработчик")
//         .about("Подписка на Geyser и выполнение транзакций")
//         .arg(
//             Arg::new("config")
//                 .short('c')
//                 .long("config")
//                 .value_name("FILE")
//                 .default_value("config.yaml")
//                 .help("Путь к файлу конфигурации"),
//         )
//         .get_matches();

//     let config_path = matches.get_one::<String>("config").unwrap();
//     let config = load_config(config_path)?;

//     let mut client = GeyserGrpcClient::build_from_shared("https://grpc.ny.shyft.to")?
//         .x_token(Some("token"))?
//         .tls_config(ClientTlsConfig::new().with_native_roots())?
//         .connect()
//         .await?;
//     let (mut subscribe_tx, mut stream) = client.subscribe().await?;

//     futures::try_join!(
//         async move {
//             subscribe_tx
//             .send(SubscribeRequest {
//                 slots: maplit::hashmap! { "".to_owned() => SubscribeRequestFilterSlots { filter_by_commitment: Some(true) } },
//                 commitment: Some(CommitmentLevel::Processed as i32),
//                 ..Default::default()
//             })
//             .await?;

//             let mut timer = interval(Duration::from_secs(3));
//             let mut id = 0;
//             loop {
//                 timer.tick().await;
//                 id += 1;
//                 subscribe_tx
//                     .send(SubscribeRequest {
//                         ping: Some(SubscribeRequestPing { id }),
//                         ..Default::default()
//                     })
//                     .await?;
//             }
//             #[allow(unreachable_code)]
//             Ok::<(), anyhow::Error>(())
//         },
//         async move {
//             while let Some(message) = stream.next().await {
//                 match message?.update_oneof.expect("valid message") {
//                     UpdateOneof::Slot(SubscribeUpdateSlot { slot, .. }) => {
//                         info!("slot received: {slot}");
//                     }
//                     UpdateOneof::Ping(_msg) => {
//                         info!("ping received");
//                     }
//                     UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
//                         info!("pong received: id#{id}");
//                     }
//                     msg => anyhow::bail!("received unexpected message: {msg:?}"),
//                 }
//             }
//             Ok::<(), anyhow::Error>(())
//         }
//     )?;

//     Ok(())
// }

use {
    clap::Parser,
    futures::{sink::SinkExt, stream::StreamExt},
    log::info,
    std::env,
    tokio::time::{interval, Duration},
    tonic::transport::channel::ClientTlsConfig,
    yellowstone_grpc_client::GeyserGrpcClient,
    yellowstone_grpc_proto::prelude::{
        subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest,
        SubscribeRequestFilterSlots, SubscribeRequestPing, SubscribeUpdatePong,
        SubscribeUpdateSlot,
    },
};

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    /// Service endpoint
    #[clap(short, long, default_value_t = String::from("http://127.0.0.1:10000"))]
    endpoint: String,

    #[clap(long)]
    x_token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var(
        env_logger::DEFAULT_FILTER_ENV,
        env::var_os(env_logger::DEFAULT_FILTER_ENV).unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    let args = Args::parse();

    let mut client = GeyserGrpcClient::build_from_shared(args.endpoint)?
        .x_token(args.x_token)?
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;
    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

    futures::try_join!(
        async move {
            subscribe_tx
            .send(SubscribeRequest {
                slots: maplit::hashmap! { "".to_owned() => SubscribeRequestFilterSlots { filter_by_commitment: Some(true) } },
                commitment: Some(CommitmentLevel::Processed as i32),
                ..Default::default()
            })
            .await?;

            let mut timer = interval(Duration::from_secs(3));
            let mut id = 0;
            loop {
                timer.tick().await;
                id += 1;
                subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id }),
                        ..Default::default()
                    })
                    .await?;
            }
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        },
        async move {
            while let Some(message) = stream.next().await {
                match message?.update_oneof.expect("valid message") {
                    UpdateOneof::Slot(SubscribeUpdateSlot { slot, .. }) => {
                        info!("slot received: {slot}");
                    }
                    UpdateOneof::Ping(_msg) => {
                        info!("ping received");
                    }
                    UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
                        info!("pong received: id#{id}");
                    }
                    msg => anyhow::bail!("received unexpected message: {msg:?}"),
                }
            }
            Ok::<(), anyhow::Error>(())
        }
    )?;

    Ok(())
}
