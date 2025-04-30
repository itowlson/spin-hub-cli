use clap::Parser;

mod commands;
mod git;
mod hub_api;
mod spin;

use commands::{CloneCommand, NewCommand, SearchCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HubCommand::parse().run().await
}

#[derive(Parser)]
#[clap(about = "Commands for using content from the Spin Up Hub")]
enum HubCommand {
    Clone(CloneCommand),
    New(NewCommand),
    // TODO: once we have more consistent surfacing of repo URLs, and can determine which samples build without intervention
    // Run(RunCommand),
    Search(SearchCommand),
}

impl HubCommand {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Clone(cmd) => cmd.run().await,
            Self::New(cmd) => cmd.run().await,
            Self::Search(cmd) => cmd.run().await,
        }
    }
}
