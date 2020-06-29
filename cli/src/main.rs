use crate::command::*;
use async_std::task;
use clap::Clap;
use exitfailure::ExitDisplay;
use ipfs_embed::{Config, Store};
use keybase_keystore::KeyStore;
use std::time::Duration;
use substrate_subxt::sp_core::sr25519;
use sunshine_client::{faucet, light, Runtime};
use sunshine_identity_cli::{key::KeySetCommand, set_device_key, Command, Error};

mod command;

#[async_std::main]
async fn main() -> Result<(), ExitDisplay<Error>> {
    Ok(run().await?)
}

type Client = sunshine_client::identity::Client<Runtime, sr25519::Pair, Store>;

async fn run() -> Result<(), Error> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let root = if let Some(root) = opts.path {
        root
    } else {
        dirs2::config_dir()
            .ok_or(Error::ConfigDirNotFound)?
            .join("sunshine")
    };
    let keystore = KeyStore::open(root.join("keystore")).await?;
    let db = sled::open(root.join("db")).unwrap();
    let db_ipfs = db.open_tree("ipfs").unwrap();
    let db_light = db.open_tree("substrate").unwrap();

    let chain_spec_bytes = include_bytes!("../../chains/staging.json");
    let chain_spec = light::ChainSpec::from_json_bytes(&chain_spec_bytes[..]).unwrap();

    let mut config = Config::from_tree(db_ipfs);
    config.network.bootstrap_nodes = chain_spec
        .boot_nodes()
        .iter()
        .map(|x| (x.multiaddr.clone(), x.peer_id.clone()))
        .collect();

    let subxt = light::build_light_client(db_light, chain_spec)
        .await
        .unwrap();
    let store = Store::new(config).unwrap();
    let client = Client::new(keystore, subxt, store);

    let mut password_changes = if client.signer().await.is_ok() {
        let sub = client.subscribe_password_changes().await?;
        client.update_password().await?;
        Some(sub)
    } else {
        None
    };

    match opts.cmd {
        SubCommand::Key(KeyCommand { cmd }) => match cmd {
            KeySubCommand::Set(KeySetCommand {
                paperkey,
                suri,
                force,
            }) => {
                let account_id = set_device_key(&client, paperkey, suri.as_deref(), force).await?;
                println!("your device key is {}", account_id.to_string());
                let amount = faucet::mint(client.subxt(), &account_id)
                    .await?
                    .unwrap()
                    .amount;
                println!("minted {} tokens into your account", amount);
                let uid = client.fetch_uid(&account_id).await?.unwrap();
                println!("your user id is {}", uid);
                Ok(())
            }
            KeySubCommand::Unlock(cmd) => cmd.exec(&client).await,
            KeySubCommand::Lock(cmd) => cmd.exec(&client).await,
        },
        SubCommand::Account(AccountCommand { cmd }) => match cmd {
            AccountSubCommand::Create(cmd) => cmd.exec(&client).await,
            AccountSubCommand::Password(cmd) => cmd.exec(&client).await,
        },
        SubCommand::Device(DeviceCommand { cmd }) => match cmd {
            DeviceSubCommand::Add(cmd) => cmd.exec(&client).await,
            DeviceSubCommand::Remove(cmd) => cmd.exec(&client).await,
            DeviceSubCommand::List(cmd) => cmd.exec(&client).await,
            DeviceSubCommand::Paperkey(cmd) => cmd.exec(&client).await,
        },
        SubCommand::Id(IdCommand { cmd }) => match cmd {
            IdSubCommand::List(cmd) => cmd.exec(&client).await,
            IdSubCommand::Prove(cmd) => cmd.exec(&client).await,
            IdSubCommand::Revoke(cmd) => cmd.exec(&client).await,
        },
        SubCommand::Wallet(WalletCommand { cmd }) => match cmd {
            WalletSubCommand::Balance(cmd) => cmd.exec(&client).await,
            WalletSubCommand::Transfer(cmd) => cmd.exec(&client).await,
        },
        SubCommand::Run => loop {
            if let Some(sub) = password_changes.as_mut() {
                if sub.next().await.is_some() {
                    client.update_password().await?;
                }
            } else {
                task::sleep(Duration::from_millis(100)).await
            }
        },
    }
}
