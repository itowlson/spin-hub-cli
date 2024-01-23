use clap::{Parser};

mod commands;
mod git;
mod hub_api;
mod spin;

use commands::{NewCommand, RunCommand, SearchCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HubCommand::parse().run().await
}

#[derive(Parser)]
#[clap(about = "Commands for using content from the Spin Up Hub")]
enum HubCommand {
    New(NewCommand),
    Run(RunCommand),
    Search(SearchCommand),
}

impl HubCommand {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::New(cmd) => cmd.run().await,
            Self::Run(cmd) => cmd.run().await,
            Self::Search(cmd) => cmd.run().await,
        }
    }
}
