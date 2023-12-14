use clap::{Parser};
use itertools::Itertools;

use crate::hub_api;

#[derive(Parser, Debug)]
#[clap(about = "Install a plugin from the Hub")]
pub struct InstallPluginCommand {
    #[clap(short = 't')]
    terms: Vec<String>,
}

impl InstallPluginCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let Some(index_entry) = self.resolve_selection().await? else {
            return Ok(());
        };

        println!("Plugin {} by {}", index_entry.title(), index_entry.author());
        println!("{}", index_entry.summary());

        let install_args = get_install_args(&index_entry);

        let spin_bin = std::env::var("SPIN_BIN_PATH").unwrap();
        let args = ["plugins", "install"].iter().chain(&install_args);
        tokio::process::Command::new(spin_bin).args(args).status().await?;

        Ok(())
    }

    async fn resolve_selection(&self) -> Result<Option<hub_api::IndexEntry>, dialoguer::Error> {
        let entries = hub_api::index().await.unwrap();
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();

        match matches.len() {
            0 => {
                println!("No plugins match your search terms");
                return Ok(None);
            }
            1 => {
                let index_entry = matches[0].clone();
                return Ok(Some(index_entry))
            },
            _ => {
                dialoguer::Select::new()
                    .with_prompt("Several plugins match your search. Use arrow keys and Enter to select, or Esc to cancel:")
                    .items(&matches.iter().map(|e| e.title()).collect_vec())
                    .interact_opt()?
                    .map(|idx| Ok(matches[idx].clone()))
                    .transpose()
            }
        }
    }

    fn is_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        self.is_terms_match(index_entry) &&
            self.is_category_match(index_entry)
    }

    fn is_terms_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        let tags = index_entry.tags();
        let title = index_entry.title_words();
        self.terms.iter()
            .map(|t| t.to_lowercase())
            .all(|t| tags.contains(&t) || title.contains(&t))
    }

    fn is_category_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        index_entry.category() == hub_api::Category::Plugin
    }
}

fn get_install_args(_index_entry: &hub_api::IndexEntry) -> Vec<&str> {
    // return either [name] or [--url, URL]
    vec!["--url", "https://github.com/fermyon/spin-trigger-sqs/releases/download/v0.5.0/trigger-sqs.json"]
}
