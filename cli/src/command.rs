use clap::Clap;
use std::path::PathBuf;
use sunshine_bounty_cli::bounty;
use sunshine_faucet_cli::MintCommand;
use sunshine_identity_cli::{account, device, id, key, wallet};

#[derive(Clone, Debug, Clap)]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: SubCommand,
    #[clap(short = "p", long = "path")]
    pub path: Option<PathBuf>,
    #[clap(short = "c", long = "chain-spec")]
    pub chain_spec: Option<PathBuf>,
}

#[derive(Clone, Debug, Clap)]
pub enum SubCommand {
    Key(KeyCommand),
    Account(AccountCommand),
    Device(DeviceCommand),
    Id(IdCommand),
    Wallet(WalletCommand),
    Bounty(BountyCommand),
    Run,
}

#[derive(Clone, Debug, Clap)]
pub struct KeyCommand {
    #[clap(subcommand)]
    pub cmd: KeySubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum KeySubCommand {
    Set(key::KeySetCommand),
    Unlock(key::KeyUnlockCommand),
    Lock(key::KeyLockCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct AccountCommand {
    #[clap(subcommand)]
    pub cmd: AccountSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum AccountSubCommand {
    Create(account::AccountCreateCommand),
    Password(account::AccountPasswordCommand),
    Mint(MintCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct DeviceCommand {
    #[clap(subcommand)]
    pub cmd: DeviceSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum DeviceSubCommand {
    Add(device::DeviceAddCommand),
    Remove(device::DeviceRemoveCommand),
    List(device::DeviceListCommand),
    Paperkey(device::DevicePaperkeyCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct IdCommand {
    #[clap(subcommand)]
    pub cmd: IdSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum IdSubCommand {
    List(id::IdListCommand),
    Prove(id::IdProveCommand),
    Revoke(id::IdRevokeCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct WalletCommand {
    #[clap(subcommand)]
    pub cmd: WalletSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum WalletSubCommand {
    Balance(wallet::WalletBalanceCommand),
    Transfer(wallet::WalletTransferCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct BountyCommand {
    #[clap(subcommand)]
    pub cmd: BountySubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum BountySubCommand {
    PostBounty(bounty::BountyPostCommand),
    ContributeToBounty(bounty::BountyContributeCommand),
    SubmitForBounty(bounty::BountySubmitCommand),
    ApproveApplication(bounty::BountyApproveCommand),
    // storage helpers
    GetBounty(bounty::GetBountyCommand),
    GetSubmission(bounty::GetSubmissionCommand),
    GetOpenBounties(bounty::GetOpenBountiesCommand),
    GetOpenSubmissions(bounty::GetOpenSubmissionsCommand),
}
