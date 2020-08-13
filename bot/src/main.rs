mod subxt;
use crate::subxt::*;
use gbot::GBot;
use ipld_block_builder::ReadonlyCache;
use std::sync::Arc;
use substrate_subxt::{sp_core::Decode, EventSubscription};
use sunshine_bounty_client::{
    bounty::{
        BountyPaymentExecutedEvent, BountyPostedEvent, BountyRaiseContributionEvent,
        BountySubmissionPostedEvent,
    },
    BountyBody,
};
use sunshine_client::{Client, Runtime};
use sunshine_client_utils::{Client as _, Result};
use tokio::task;

pub struct Bot {
    pub client: Client,
    pub bounty_post_sub: EventSubscription<Runtime>,
    pub bounty_contrib_sub: EventSubscription<Runtime>,
    pub bounty_submit_sub: EventSubscription<Runtime>,
    pub bounty_approval_sub: EventSubscription<Runtime>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let github_bot = GBot::new()?;
    let root = dirs::config_dir().unwrap().join("sunshine-bounty-bot");
    let client = Client::new(&root, None).await?;
    // subscribe to bounty posts
    let mut bounty_post_sub = bounty_post_subscriber(&client).await?;
    // subscribe to bounty contributions
    let mut bounty_contrib_sub = bounty_contribution_subscriber(&client).await?;
    // subscribe to bounty submissions
    let mut bounty_submit_sub = bounty_submission_subscriber(&client).await?;
    // subscribe to bounty payments
    let mut bounty_approval_sub = bounty_approval_subscriber(&client).await?;
    let shared_client = Arc::new(client);
    // run github bot
    loop {
        let bp_sub = task::spawn(poll_bounty_postings(
            bounty_post_sub,
            shared_client.clone(),
            github_bot.clone(),
        ));
        let bc_sub = task::spawn(poll_bounty_contributions(
            bounty_contrib_sub,
            shared_client.clone(),
            github_bot.clone(),
        ));
        let bs_sub = task::spawn(poll_bounty_submissions(
            bounty_submit_sub,
            shared_client.clone(),
            github_bot.clone(),
        ));
        let ba_sub = task::spawn(poll_bounty_approvals(
            bounty_approval_sub,
            shared_client.clone(),
            github_bot.clone(),
        ));
        bounty_post_sub = bp_sub.await??;
        bounty_contrib_sub = bc_sub.await??;
        bounty_submit_sub = bs_sub.await??;
        bounty_approval_sub = ba_sub.await??;
    }
}

async fn poll_bounty_postings(
    mut event_sub: EventSubscription<Runtime>,
    client: Arc<Client>,
    github: GBot,
) -> Result<EventSubscription<Runtime>> {
    loop {
        if let Some(Ok(raw)) = event_sub.next().await {
            // get event data
            let event = BountyPostedEvent::<Runtime>::decode(&mut &raw.data[..])?;
            // fetch structured data from client
            let event_cid = event.description.to_cid()?;
            let bounty_body: BountyBody = client.offchain_client().get(&event_cid).await?;
            // issue comment
            github
                .issue_comment_bounty_post(
                    event.amount,
                    event.id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        } else {
            break;
        }
    }
    Ok(event_sub)
}

async fn poll_bounty_contributions(
    mut event_sub: EventSubscription<Runtime>,
    client: Arc<Client>,
    github: GBot,
) -> Result<EventSubscription<Runtime>> {
    loop {
        if let Some(Ok(raw)) = event_sub.next().await {
            // get event data
            let event = BountyRaiseContributionEvent::<Runtime>::decode(&mut &raw.data[..])?;
            // fetch structured data from client
            let event_cid = event.bounty_ref.to_cid()?;
            let bounty_body: BountyBody = client.offchain_client().get(&event_cid).await?;
            // issue comment
            github
                .issue_comment_bounty_contribute(
                    event.amount,
                    event.total,
                    event.bounty_id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        } else {
            break;
        }
    }
    Ok(event_sub)
}

async fn poll_bounty_submissions(
    mut event_sub: EventSubscription<Runtime>,
    client: Arc<Client>,
    github: GBot,
) -> Result<EventSubscription<Runtime>> {
    loop {
        if let Some(Ok(raw)) = event_sub.next().await {
            // get event data
            let event = BountySubmissionPostedEvent::<Runtime>::decode(&mut &raw.data[..])?;
            // fetch structured data from client
            let bounty_event_cid = event.bounty_ref.to_cid()?;
            let submission_event_cid = event.submission_ref.to_cid()?;
            let bounty_body: BountyBody = client.offchain_client().get(&bounty_event_cid).await?;
            let submission_body: BountyBody =
                client.offchain_client().get(&submission_event_cid).await?;
            // issue comment
            github
                .issue_comment_bounty_submission(
                    event.amount,
                    event.bounty_id,
                    event.id,
                    submission_body.repo_owner,
                    submission_body.repo_name,
                    submission_body.issue_number,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        } else {
            break;
        }
    }
    Ok(event_sub)
}

async fn poll_bounty_approvals(
    mut event_sub: EventSubscription<Runtime>,
    client: Arc<Client>,
    github: GBot,
) -> Result<EventSubscription<Runtime>> {
    loop {
        if let Some(Ok(raw)) = event_sub.next().await {
            // get event data
            let event = BountyPaymentExecutedEvent::<Runtime>::decode(&mut &raw.data[..])?;
            // fetch structured data from client
            let bounty_event_cid = event.bounty_ref.to_cid()?;
            let submission_event_cid = event.submission_ref.to_cid()?;
            let bounty_body: BountyBody = client.offchain_client().get(&bounty_event_cid).await?;
            let submission_body: BountyBody =
                client.offchain_client().get(&submission_event_cid).await?;
            // issue comment
            github
                .issue_comment_submission_approval(
                    event.amount,
                    event.new_total,
                    event.submission_id,
                    event.bounty_id,
                    submission_body.repo_owner,
                    submission_body.repo_name,
                    submission_body.issue_number,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        } else {
            break;
        }
    }
    Ok(event_sub)
}
