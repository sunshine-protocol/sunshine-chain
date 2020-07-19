use clap::Clap;
use std::path::PathBuf;
use sunshine_bounty_cli::{bounty, org, shares, vote};
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
    Org(OrgCommand),
    Vote(VoteCommand),
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
pub struct OrgCommand {
    #[clap(subcommand)]
    pub cmd: OrgSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum OrgSubCommand {
    // share stuff
    IssueShares(shares::SharesIssueCommand),
    BurnShares(shares::SharesBurnCommand),
    BatchIssueShares(shares::SharesBatchIssueCommand),
    BatchBurnShares(shares::SharesBatchBurnCommand),
    ReserveShares(shares::SharesReserveCommand),
    UnreserveShares(shares::SharesUnReserveCommand),
    LockShares(shares::SharesLockCommand),
    UnlockShares(shares::SharesUnLockCommand),
    // full org stuff
    RegisterFlatOrg(org::OrgRegisterFlatCommand),
    RegisterWeightedOrg(org::OrgRegisterWeightedCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct VoteCommand {
    #[clap(subcommand)]
    pub cmd: VoteSubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum VoteSubCommand {
    CreateSignalThresholdVote(vote::VoteCreateSignalThresholdCommand),
    CreatePercentThresholdVote(vote::VoteCreatePercentThresholdCommand),
    CreateUnanimousConsentVote(vote::VoteCreateUnanimousConsentCommand),
    SubmitVote(vote::VoteSubmitCommand),
}

#[derive(Clone, Debug, Clap)]
pub struct BountyCommand {
    #[clap(subcommand)]
    pub cmd: BountySubCommand,
}

#[derive(Clone, Debug, Clap)]
pub enum BountySubCommand {
    PostBounty(bounty::BountyPostCommand),
    ApplyForBounty(bounty::BountyApplicationCommand),
    TriggerApplicationReview(bounty::BountyTriggerApplicationReviewCommand),
    SudoApproveApplication(bounty::BountySudoApproveApplicationCommand),
    PollApplication(bounty::BountyPollApplicationCommand),
    SubmitMilestone(bounty::BountySubmitMilestoneCommand),
    TriggerMilestoneReview(bounty::BountyTriggerMilestoneReviewCommand),
    SudoApproveMilestone(bounty::BountySudoApproveMilestoneCommand),
    PollMilestone(bounty::BountyPollMilestoneCommand),
}
