use crate::command::*;
use async_std::task;
use clap::Clap;
use exitfailure::ExitDisplay;
use std::time::Duration;
use sunshine_client::{identity::IdentityClient, Client, Error as ClientError};
use sunshine_core::{ChainClient, Keystore};
use sunshine_faucet_cli::{Command as _, MintCommand};
use sunshine_identity_cli::{key::KeySetCommand, set_device_key, Command as _};

mod command;
mod error;
use error::Error;

#[async_std::main]
async fn main() -> Result<(), ExitDisplay<Error<ClientError>>> {
    Ok(run().await?)
}

async fn run() -> Result<(), Error<ClientError>> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let root = if let Some(root) = opts.path {
        root
    } else {
        dirs::config_dir()
            .ok_or(Error::ConfigDirNotFound)?
            .join("sunshine")
    };

    let mut client = Client::new(&root, opts.chain_spec.as_deref())
        .await
        .map_err(Error::Client)?;

    let mut password_changes = if client.keystore().chain_signer().is_some() {
        let sub = client
            .subscribe_password_changes()
            .await
            .map_err(Error::Client)?;
        client.update_password().await.map_err(Error::Client)?;
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
                let account_id =
                    set_device_key(&mut client, paperkey, suri.as_deref(), force).await?;
                println!("your device key is {}", account_id.to_string());
                MintCommand.exec(&mut client).await.map_err(Error::Client)?;
                let uid = client
                    .fetch_uid(&account_id)
                    .await
                    .map_err(Error::Client)?
                    .unwrap();
                println!("your user id is {}", uid);
                Ok(())
            }
            KeySubCommand::Unlock(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            KeySubCommand::Lock(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
        },
        SubCommand::Account(AccountCommand { cmd }) => match cmd {
            AccountSubCommand::Create(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            AccountSubCommand::Password(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Identity)
            }
            AccountSubCommand::Mint(cmd) => cmd.exec(&mut client).await.map_err(Error::Client),
        },
        SubCommand::Device(DeviceCommand { cmd }) => match cmd {
            DeviceSubCommand::Add(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            DeviceSubCommand::Remove(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            DeviceSubCommand::List(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            DeviceSubCommand::Paperkey(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
        },
        SubCommand::Id(IdCommand { cmd }) => match cmd {
            IdSubCommand::List(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            IdSubCommand::Prove(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            IdSubCommand::Revoke(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
        },
        SubCommand::Wallet(WalletCommand { cmd }) => match cmd {
            WalletSubCommand::Balance(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
            WalletSubCommand::Transfer(cmd) => cmd.exec(&mut client).await.map_err(Error::Identity),
        },
        SubCommand::Org(OrgCommand { cmd }) => match cmd {
            OrgSubCommand::IssueShares(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            OrgSubCommand::BurnShares(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            OrgSubCommand::BatchIssueShares(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            OrgSubCommand::BatchBurnShares(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            OrgSubCommand::ReserveShares(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            OrgSubCommand::UnreserveShares(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            OrgSubCommand::LockShares(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            OrgSubCommand::UnlockShares(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            OrgSubCommand::RegisterFlatOrg(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            OrgSubCommand::RegisterWeightedOrg(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
        },
        SubCommand::Vote(VoteCommand { cmd }) => match cmd {
            VoteSubCommand::CreateSignalThresholdVote(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            VoteSubCommand::CreatePercentThresholdVote(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            VoteSubCommand::CreateUnanimousConsentVote(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            VoteSubCommand::SubmitVote(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
        },
        SubCommand::Bounty(BountyCommand { cmd }) => match cmd {
            BountySubCommand::PostBounty(cmd) => cmd.exec(&mut client).await.map_err(Error::Bounty),
            BountySubCommand::ApplyForBounty(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::TriggerApplicationReview(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::SudoApproveApplication(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::PollApplication(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::SubmitMilestone(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::TriggerMilestoneReview(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::SudoApproveMilestone(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
            BountySubCommand::PollMilestone(cmd) => {
                cmd.exec(&mut client).await.map_err(Error::Bounty)
            }
        },
        SubCommand::Run => loop {
            if let Some(sub) = password_changes.as_mut() {
                if sub.next().await.is_some() {
                    client.update_password().await.map_err(Error::Client)?;
                }
            } else {
                task::sleep(Duration::from_millis(100)).await
            }
        },
    }
}
