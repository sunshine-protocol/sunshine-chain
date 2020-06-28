use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::path::PathBuf;
use std::str::FromStr;
use sunshine_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SystemConfig,
    WASM_BINARY,
};

pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

fn seed_to_public<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&seed, None).unwrap().public()
}

fn seed_to_account_id<TPublic: Public>(seed: &str) -> AccountId
where
    <TPublic::Pair as Pair>::Public: Into<<Signature as Verify>::Signer>,
{
    seed_to_public::<TPublic>(seed).into().into_account()
}

fn seed_to_authority_keys(seed: &str) -> (AuraId, GrandpaId) {
    (
        seed_to_public::<AuraId>(seed),
        seed_to_public::<GrandpaId>(seed),
    )
}

#[derive(Clone, Debug)]
pub enum Chain {
    Dev,
    Local,
    Json(PathBuf),
}

impl Chain {
    pub fn to_chain_spec(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Self::Dev => dev_chain_spec(),
            Self::Local => local_chain_spec(),
            Self::Json(path) => ChainSpec::from_json_file(path)?,
        })
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(chain: &str) -> Result<Self, Self::Err> {
        Ok(match chain {
            "dev" => Chain::Dev,
            "" | "local" => Chain::Local,
            path => Chain::Json(PathBuf::from(path)),
        })
    }
}

pub fn dev_chain_spec() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Development,
        || {
            testnet_genesis(
                &[seed_to_authority_keys("//Alice")],
                &[
                    seed_to_account_id::<sr25519::Public>("//Alice"),
                    seed_to_account_id::<sr25519::Public>("//Alice//stash"),
                ],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

pub fn local_chain_spec() -> ChainSpec {
    ChainSpec::from_genesis(
        "Local Testnet",
        "local_testnet",
        ChainType::Local,
        || {
            testnet_genesis(
                &[
                    seed_to_authority_keys("//Alice"),
                    seed_to_authority_keys("//Bob"),
                ],
                &[
                    seed_to_account_id::<sr25519::Public>("//Alice"),
                    seed_to_account_id::<sr25519::Public>("//Alice//stash"),
                    seed_to_account_id::<sr25519::Public>("//Bob"),
                    seed_to_account_id::<sr25519::Public>("//Bob//stash"),
                ],
            )
        },
        vec![],
        None,
        None,
        None,
        None,
    )
}

fn testnet_genesis(
    initial_authorities: &[(AuraId, GrandpaId)],
    endowed_accounts: &[AccountId],
) -> GenesisConfig {
    GenesisConfig {
        frame_system: Some(SystemConfig {
            code: WASM_BINARY.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
    }
}
