use sc_cli::{RunCmd, Subcommand, SubstrateCli};
use std::str::FromStr;
use structopt::StructOpt;
use sunshine_node::{chain_spec::Chain, new_full_start, service};

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,
}

impl SubstrateCli for Cli {
    fn impl_name() -> &'static str {
        sunshine_node::IMPL_NAME
    }

    fn impl_version() -> &'static str {
        sunshine_node::IMPL_VERSION
    }

    fn description() -> &'static str {
        sunshine_node::DESCRIPTION
    }

    fn author() -> &'static str {
        sunshine_node::AUTHOR
    }

    fn support_url() -> &'static str {
        sunshine_node::SUPPORT_URL
    }

    fn copyright_start_year() -> i32 {
        sunshine_node::COPYRIGHT_START_YEAR
    }

    fn executable_name() -> &'static str {
        sunshine_node::EXECUTABLE_NAME
    }

    fn load_spec(&self, chain: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(Box::new(Chain::from_str(chain)?.to_chain_spec()?))
    }
}

fn main() -> sc_cli::Result<()> {
    let cli = <Cli as SubstrateCli>::from_args();

    match &cli.subcommand {
        Some(subcommand) => {
            let runner = cli.create_runner(subcommand)?;
            runner.run_subcommand(subcommand, |config| Ok(new_full_start!(config).0))
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node(
                service::new_light,
                service::new_full,
                sunshine_runtime::VERSION,
            )
        }
    }
}
