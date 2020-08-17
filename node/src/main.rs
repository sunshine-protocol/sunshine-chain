use sc_cli::{RunCmd, Runner, RuntimeVersion, Subcommand, SubstrateCli};
use sc_service::{ChainSpec, DatabaseConfig, Role, ServiceParams};
use structopt::StructOpt;

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

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(sunshine_node::dev_chain_spec()),
            "" | "local" => Box::new(sunshine_node::local_chain_spec()),
            path => Box::new(sunshine_node::ChainSpec::from_json_file(path.into())?),
        })
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &sunshine_runtime::VERSION
    }
}

fn main() -> sc_cli::Result<()> {
    let cli = <Cli as SubstrateCli>::from_args();
    match &cli.subcommand {
        Some(subcommand) => {
            let mut runner = cli.create_runner(subcommand)?;
            force_parity_db(&mut runner);
            runner.run_subcommand(subcommand, |config| {
                let ServiceParams {
                    client,
                    backend,
                    task_manager,
                    import_queue,
                    ..
                } = sunshine_node::new_full_params(config)?.0;
                Ok((client, backend, import_queue, task_manager))
            })
        }
        None => {
            let mut runner = cli.create_runner(&cli.run)?;
            force_parity_db(&mut runner);
            runner.run_node_until_exit(|config| {
                match config.role {
                    Role::Light => sunshine_node::new_light(config),
                    _ => sunshine_node::new_full(config),
                }
                .map(|service| service.0)
            })
        }
    }
}

fn force_parity_db(runner: &mut Runner<Cli>) {
    let config = runner.config_mut();
    let path = config.database.path().unwrap().to_path_buf();
    config.database = DatabaseConfig::ParityDb { path };
}
