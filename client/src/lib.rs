use ipfs_embed::{Config, Store};
use keybase_keystore::KeyStore;
use std::path::Path;
use substrate_subxt::balances::{AccountData, Balances};
use substrate_subxt::sp_runtime::traits::{IdentifyAccount, Verify};
use substrate_subxt::system::System;
use substrate_subxt::{sp_core, sp_runtime, ClientBuilder};
use sunshine_faucet_client::Faucet;
use sunshine_identity_client::{Client, Identity};
use sunshine_identity_utils::cid::CidBytes;
use thiserror::Error;

pub use substrate_subxt::balances;
pub use substrate_subxt::system;
pub use sunshine_faucet_client as faucet;
pub use sunshine_identity_client as identity;
#[cfg(feature = "light")]
pub mod light;

pub type AccountId = <<sp_runtime::MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Uid = u32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runtime;

impl System for Runtime {
    type Index = u32;
    type BlockNumber = u32;
    type Hash = sp_core::H256;
    type Hashing = sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Address = AccountId;
    type Header = sp_runtime::generic::Header<Self::BlockNumber, Self::Hashing>;
    type Extrinsic = sp_runtime::OpaqueExtrinsic;
    type AccountData = ();
}

impl Balances for Runtime {
    type Balance = u128;
}

impl Identity for Runtime {
    type Uid = Uid;
    type Cid = CidBytes;
    type Mask = [u8; 32];
    type Gen = u16;
    type IdAccountData = AccountData<<Self as Balances>::Balance>;
}

impl Faucet for Runtime {}

impl substrate_subxt::Runtime for Runtime {
    type Signature = sp_runtime::MultiSignature;
    type Extra = substrate_subxt::DefaultExtra<Self>;
}

#[cfg(not(feature = "light"))]
pub async fn build_client(
    root: &Path,
) -> Result<Client<Runtime, sp_core::sr25519::Pair, Store>, Error> {
    let keystore = KeyStore::open(root.join("keystore")).await?;
    let db = sled::open(root.join("db"))?;
    let db_ipfs = db.open_tree("ipfs")?;

    let config = Config::from_tree(db_ipfs);
    let subxt = ClientBuilder::new().build().await?;
    let store = Store::new(config)?;
    Ok(Client::new(keystore, subxt, store))
}

#[cfg(feature = "light")]
pub async fn build_client(
    root: &Path,
) -> Result<Client<Runtime, sp_core::sr25519::Pair, Store>, Error> {
    let keystore = KeyStore::open(root.join("keystore")).await?;
    let db = sled::open(root.join("db"))?;
    let db_ipfs = db.open_tree("ipfs")?;
    let db_light = db.open_tree("substrate")?;

    let chain_spec_bytes = include_bytes!("../../chains/staging.json");
    let chain_spec =
        light::ChainSpec::from_json_bytes(&chain_spec_bytes[..]).map_err(ChainSpecError)?;

    let mut config = Config::from_tree(db_ipfs);
    config.network.bootstrap_nodes = chain_spec
        .boot_nodes()
        .iter()
        .map(|x| (x.multiaddr.clone(), x.peer_id.clone()))
        .collect();

    let light_client = light::build_light_client(db_light, chain_spec).await?;
    let subxt = ClientBuilder::new()
        .set_client(light_client)
        .build()
        .await?;
    let store = Store::new(config)?;
    Ok(Client::new(keystore, subxt, store))
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Sled(#[from] sled::Error),
    #[error(transparent)]
    Ipfs(#[from] ipfs_embed::Error),
    #[error(transparent)]
    Keystore(#[from] keybase_keystore::Error),
    #[error(transparent)]
    Subxt(#[from] substrate_subxt::Error),
    #[cfg(feature = "light")]
    #[error(transparent)]
    Light(#[from] sc_service::Error),
    #[cfg(feature = "light")]
    #[error(transparent)]
    ChainSpec(#[from] ChainSpecError),
}

#[derive(Debug, Error)]
#[error("{0}")]
pub struct ChainSpecError(String);
