use anyhow::anyhow;
use clap::Parser;
use itertools::Itertools;

use crate::{hub_api, git};

#[derive(Parser, Debug)]
#[clap(about = "Clone a sample from the Hub")]
pub struct CloneCommand {
    #[clap(short = 't')]
    terms: Vec<String>,
}

impl CloneCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let Some(index_entry) = self.resolve_selection().await? else {
            return Ok(());
        };

        println!("Sample {} by {}", index_entry.title(), index_entry.author());
        println!("{}", index_entry.summary());

        if !dialoguer::Confirm::new().with_prompt("Clone this sample?").default(true).interact_opt()?.unwrap_or_default() {
            return Ok(());
        }

        let repo = get_repo(&index_entry)?;

        git::clone_decoupled(&repo).await?;

        let clone_dir = git::clone_dir(&repo)?;

        println!("Sample cloned to {clone_dir}");

        Ok(())
    }

    async fn resolve_selection(&self) -> Result<Option<hub_api::IndexEntry>, dialoguer::Error> {
        let entries = hub_api::index().await.unwrap();
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();

        match matches.len() {
            0 => {
                println!("No templates matches your search terms");
                return Ok(None);
            }
            1 => {
                let index_entry = matches[0].clone();
                return Ok(Some(index_entry))
            },
            _ => {
                dialoguer::Select::new()
                    .with_prompt("Several templates match your search. Use arrow keys and Enter to select, or Esc to cancel:")
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
        index_entry.category() == hub_api::Category::Sample
    }
}

fn get_repo(index_entry: &hub_api::IndexEntry) -> anyhow::Result<&String> {
    index_entry.repo_url().ok_or_else(|| anyhow!("Hub entry {} is not a cloneable sample (no repository URL). Open {} in your browser and follow the guidance there.", index_entry.title(), index_entry.url()))
}
