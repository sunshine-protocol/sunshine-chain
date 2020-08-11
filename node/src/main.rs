use sc_cli::{Database, RunCmd, RuntimeVersion, Subcommand, SubstrateCli};
use sc_service::{ChainSpec, Role, ServiceParams};
use std::str::FromStr;
use structopt::StructOpt;
use sunshine_node::{chain_spec::Chain, service};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        sunshine_node::IMPL_NAME.into()
    }

    fn impl_version() -> String {
        sunshine_node::IMPL_VERSION.into()
    }

    fn description() -> String {
        sunshine_node::DESCRIPTION.into()
    }

    fn author() -> String {
        sunshine_node::AUTHOR.into()
    }

    fn support_url() -> String {
        sunshine_node::SUPPORT_URL.into()
    }

    fn copyright_start_year() -> i32 {
        sunshine_node::COPYRIGHT_START_YEAR
    }

    fn executable_name() -> String {
        sunshine_node::EXECUTABLE_NAME.into()
    }

    fn load_spec(&self, chain: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(Box::new(Chain::from_str(chain)?.to_chain_spec()?))
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &sunshine_runtime::VERSION
    }
}

fn main() -> sc_cli::Result<()> {
    let mut cli = <Cli as SubstrateCli>::from_args();
    let db = cli
        .run
        .import_params
        .database_params
        .database
        .unwrap_or(Database::ParityDb);
    cli.run.import_params.database_params.database = Some(db);

    match &cli.subcommand {
        Some(subcommand) => {
            let runner = cli.create_runner(subcommand)?;
            runner.run_subcommand(subcommand, |config| {
                let ServiceParams {
                    client,
                    backend,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_full_params(config)?.0;
                Ok((client, backend, import_queue, task_manager))
            })
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| {
                match config.role {
                    Role::Light => service::new_light(config),
                    _ => service::new_full(config),
                }
                .map(|service| service.0)
            })
        }
    }
}
