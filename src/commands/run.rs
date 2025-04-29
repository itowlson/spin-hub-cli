use std::path::PathBuf;

use clap::Parser;
use itertools::Itertools;

use crate::{hub_api, git};

#[derive(Parser, Debug)]
#[clap(about = "Create an application from a template on the Hub")]
pub struct RunCommand {
    #[clap(short = 't')]
    terms: Vec<String>,

    #[clap(long = "deploy")]
    deploy: bool,
}

impl RunCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let Some(index_entry) = self.resolve_selection().await? else {
            return Ok(());
        };

        println!("Template {} by {}", index_entry.title(), index_entry.author());
        println!("{}", index_entry.summary());

        let prompt = if self.deploy {
            "Clone and deploy this sample?"
        } else {
            "Clone and run this sample?"
        };
        if !dialoguer::Confirm::new().with_prompt(prompt).default(true).interact_opt()?.unwrap_or_default() {
            return Ok(());
        }

        let (repo, manifest_path) = get_repo_and_manifest_path(&index_entry)?;

        git::clone_decoupled(&repo).await?;

        let clone_dir = git::clone_dir(&repo)?;

        let manifest_path = PathBuf::from(&clone_dir).join(manifest_path);

        // TODO: DAMMIT THERE ARE SO MANY PIPENVS AND NPMS AND STUFF

        let verb = if self.deploy {
            "deploy"
        } else {
            "up"
        };
        
        let mut child = crate::spin::bin()
            .arg(verb)
            .args(["--build", "-f"])
            .arg(manifest_path)
            .spawn()?;

        _ = child.wait().await;

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

fn get_repo_and_manifest_path(_: &hub_api::IndexEntry) -> anyhow::Result<(String, String)> {
    // TODO: this
    Ok(("https://github.com/mikkelhegn/redirect".to_owned(), "spin.toml".to_owned()))
}
