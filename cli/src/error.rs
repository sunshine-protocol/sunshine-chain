use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error<E: std::fmt::Debug + std::error::Error + 'static> {
    #[error(transparent)]
    Client(E),
    #[error("Failed to find config dir. Use `--path` to supply a suitable directory.")]
    ConfigDirNotFound,
    #[error(transparent)]
    Identity(#[from] sunshine_identity_cli::Error<E>),
    #[error(transparent)]
    Bounty(#[from] sunshine_bounty_cli::Error<E>),
}
