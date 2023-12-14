use clap::{Parser};

mod commands;
mod hub_api;
mod spin;

use commands::{InstallPluginCommand, NewCommand, SearchCommand};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HubCommand::parse().run().await
}

#[derive(Parser)]
#[clap(about = "Commands for using content from the Spin Up Hub")]
enum HubCommand {
    InstallPlugin(InstallPluginCommand),
    New(NewCommand),
    Search(SearchCommand),
}

impl HubCommand {
    async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::InstallPlugin(cmd) => cmd.run().await,
            Self::New(cmd) => cmd.run().await,
            Self::Search(cmd) => cmd.run().await,
        }
    }
}
