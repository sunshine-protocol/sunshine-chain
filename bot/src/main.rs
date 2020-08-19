use ipld_block_builder::ReadonlyCache;
use std::marker::PhantomData;
use std::sync::Arc;
use substrate_subxt as subxt;
use sunshine_bounty_gbot::GBot;
use sunshine_client::client::{Client as _, Result};
use sunshine_client::{
    bounty::{
        Bounty, BountyEventsDecoder, BountyPaymentExecutedEvent, BountyPostedEvent,
        BountyRaiseContributionEvent, BountySubmissionPostedEvent,
    },
    GithubIssue,
};
use sunshine_client::{Client, Runtime};
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let github = GBot::new()?;
    let root = dirs::config_dir().unwrap().join("sunshine-bounty-bot");
    let client = Arc::new(Client::new(&root, "ws://127.0.0.1:9944").await?);

    let post =
        Subscription::<_, BountyPostedEvent<Runtime>>::subscribe(client.chain_client()).await?;
    let contrib =
        Subscription::<_, BountyRaiseContributionEvent<Runtime>>::subscribe(client.chain_client())
            .await?;
    let submit =
        Subscription::<_, BountySubmissionPostedEvent<Runtime>>::subscribe(client.chain_client())
            .await?;
    let approval =
        Subscription::<_, BountyPaymentExecutedEvent<Runtime>>::subscribe(client.chain_client())
            .await?;

    let post = task::spawn(process_subscription(client.clone(), github.clone(), post));
    let contrib = task::spawn(process_subscription(
        client.clone(),
        github.clone(),
        contrib,
    ));
    let submit = task::spawn(process_subscription(client.clone(), github.clone(), submit));
    let approval = task::spawn(process_subscription(client, github, approval));

    post.await?;
    contrib.await?;
    submit.await?;
    approval.await?;

    Ok(())
}

pub struct Subscription<R: subxt::Runtime, E: subxt::Event<R>> {
    _marker: PhantomData<E>,
    subscription: subxt::EventSubscription<R>,
    open: bool,
}

impl<R: subxt::Runtime + Bounty, E: subxt::Event<R>> Subscription<R, E> {
    async fn subscribe(client: &subxt::Client<R>) -> Result<Self> {
        let sub = client.subscribe_events().await?;
        let mut decoder = subxt::EventsDecoder::<R>::new(client.metadata().clone());
        decoder.with_bounty();
        let mut subscription = subxt::EventSubscription::<R>::new(sub, decoder);
        subscription.filter_event::<E>();
        Ok(Self {
            _marker: PhantomData,
            subscription,
            open: true,
        })
    }

    pub fn close(&mut self) {
        self.open = false;
    }
    fn closed(&self) -> bool {
        !self.open
    }

    async fn next(&mut self) -> Option<Result<E>> {
        match self.subscription.next().await {
            Some(Ok(raw)) => Some(E::decode(&mut &raw.data[..]).map_err(Into::into)),
            Some(Err(err)) => Some(Err(err.into())),
            None => None,
        }
    }
}

async fn process_subscription<E: subxt::Event<Runtime> + Into<Event>>(
    client: Arc<Client>,
    github: GBot,
    mut subscription: Subscription<Runtime, E>,
) {
    loop {
        if subscription.closed() {
            // occurs if `close` is called on the subscription
            break;
        } else if let Some(res) = subscription.next().await {
            if let Err(err) = process_event(&client, &github, res.map(Into::into)).await {
                log::error!("{:?}", err);
            }
        } else {
            // should never happen
            break;
        }
    }
}

pub enum Event {
    BountyPosted(BountyPostedEvent<Runtime>),
    RaiseContribution(BountyRaiseContributionEvent<Runtime>),
    SubmissionPosted(BountySubmissionPostedEvent<Runtime>),
    PaymentExecuted(BountyPaymentExecutedEvent<Runtime>),
}

impl From<BountyPostedEvent<Runtime>> for Event {
    fn from(ev: BountyPostedEvent<Runtime>) -> Self {
        Self::BountyPosted(ev)
    }
}

impl From<BountyRaiseContributionEvent<Runtime>> for Event {
    fn from(ev: BountyRaiseContributionEvent<Runtime>) -> Self {
        Self::RaiseContribution(ev)
    }
}

impl From<BountySubmissionPostedEvent<Runtime>> for Event {
    fn from(ev: BountySubmissionPostedEvent<Runtime>) -> Self {
        Self::SubmissionPosted(ev)
    }
}

impl From<BountyPaymentExecutedEvent<Runtime>> for Event {
    fn from(ev: BountyPaymentExecutedEvent<Runtime>) -> Self {
        Self::PaymentExecuted(ev)
    }
}

async fn process_event(client: &Client, github: &GBot, event: Result<Event>) -> Result<()> {
    match event? {
        Event::BountyPosted(event) => {
            // fetch structured data from client
            let event_cid = event.description.to_cid()?;
            let bounty_body: GithubIssue = client.offchain_client().get(&event_cid).await?;
            // new issue comment
            github
                .new_bounty_issue(
                    event.amount,
                    event.id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        }
        Event::RaiseContribution(event) => {
            // fetch structured data from client
            let event_cid = event.bounty_ref.to_cid()?;
            let bounty_body: GithubIssue = client.offchain_client().get(&event_cid).await?;
            // update existing bounty comment
            github
                .update_bounty_issue(
                    event.total,
                    event.bounty_id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        }
        Event::SubmissionPosted(event) => {
            // fetch structured data from client
            let bounty_event_cid = event.bounty_ref.to_cid()?;
            let submission_event_cid = event.submission_ref.to_cid()?;
            let bounty_body: GithubIssue = client.offchain_client().get(&bounty_event_cid).await?;
            let submission_body: GithubIssue =
                client.offchain_client().get(&submission_event_cid).await?;
            // new issue comment
            github
                .new_submission_issue(
                    event.amount,
                    event.bounty_id,
                    event.id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                    submission_body.repo_owner,
                    submission_body.repo_name,
                    submission_body.issue_number,
                )
                .await?;
        }
        Event::PaymentExecuted(event) => {
            // fetch structured data from client
            let bounty_event_cid = event.bounty_ref.to_cid()?;
            let submission_event_cid = event.submission_ref.to_cid()?;
            let bounty_body: GithubIssue = client.offchain_client().get(&bounty_event_cid).await?;
            let submission_body: GithubIssue =
                client.offchain_client().get(&submission_event_cid).await?;
            // update existing submission comment
            github
                .approve_submission_issue(
                    event.amount,
                    event.bounty_id,
                    event.submission_id,
                    bounty_body.repo_owner.clone(),
                    bounty_body.repo_name.clone(),
                    bounty_body.issue_number,
                    submission_body.repo_owner,
                    submission_body.repo_name,
                    submission_body.issue_number,
                )
                .await?;
            // update existing bounty comment
            github
                .update_bounty_issue(
                    event.new_total,
                    event.bounty_id,
                    bounty_body.repo_owner,
                    bounty_body.repo_name,
                    bounty_body.issue_number,
                )
                .await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{process_subscription, Subscription};
    use std::sync::Arc;
    use sunshine_bounty_gbot::GBot;
    use sunshine_client::{
        bounty::{BountyClient, BountyPostedEvent},
        client::Client as _,
        mock::AccountKeyring,
        Client, GithubIssue, Runtime,
    };
    use sunshine_crypto::{keychain::TypedPair, secrecy::SecretString};

    #[tokio::test]
    async fn bounty_post_test() {
        env_logger::init();
        let github = GBot::new().expect("Initialize Github Env Var");
        let alice_root = dirs::config_dir().unwrap().join("demo-alice");
        let mut alice_client = Client::new(&alice_root, "ws://127.0.0.1:9944")
            .await
            .expect("Must Connect to Running Node");
        alice_client
            .set_key(
                TypedPair::from_suri(&AccountKeyring::Alice.to_seed()).unwrap(),
                &SecretString::new("password".to_string()),
                true,
            )
            .await
            .expect("Keystore Must Be Available");
        alice_client
            .unlock(&SecretString::new("password".to_string()))
            .await
            .expect("Keystore Must Be Available");
        let arc_alice = Arc::new(alice_client);
        // run bot post subscriber
        let mut post = Arc::new(
            Subscription::<_, BountyPostedEvent<Runtime>>::subscribe(arc_alice.chain_client())
                .await
                .unwrap(),
        );
        let post_task = tokio::task::spawn(process_subscription(
            arc_alice.clone(),
            github.clone(),
            *(post.clone()),
        ));
        let bounty = GithubIssue {
            repo_owner: "sunshine-protocol".to_string(),
            repo_name: "sunshine".to_string(),
            issue_number: 8,
        };
        let event = arc_alice.clone().post_bounty(bounty, 10u128).await.unwrap();
        let expected_event = BountyPostedEvent {
            depositer: AccountKeyring::Alice.to_account_id(),
            amount: 10,
            id: 1,
            description: event.description.clone(),
        };
        assert_eq!(event, expected_event);
        post_task.await.unwrap();
        // attempt to close post subscriber to end test scope
        *post.close();
    }
}
