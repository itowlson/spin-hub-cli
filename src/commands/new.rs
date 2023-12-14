use std::path::PathBuf;

use clap::{Parser};
use itertools::Itertools;

use crate::hub_api;

#[derive(Parser, Debug)]
#[clap(about = "Create an application from a template on the Hub")]
pub struct NewCommand {
    #[clap(short = 't')]
    terms: Vec<String>,

    name: String,
}

impl NewCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let Some(index_entry) = self.resolve_selection().await? else {
            return Ok(());
        };

        println!("Template {} by {}", index_entry.title(), index_entry.author());
        println!("{}", index_entry.summary());

        let (repo, id) = get_repo_and_id(&index_entry)?;

        self.run_template(repo, id).await
    }

    async fn run_template(&self, repo: String, id: String) -> anyhow::Result<()> {
        use spin_templates::*;

        let tempdir = tempfile::tempdir().unwrap();
        let template_manager = spin_templates::TemplateManager::in_dir(tempdir.path());

        let source = TemplateSource::try_from_git(&repo, &None, &crate::spin::version())?;
        let options = InstallOptions::default();
        template_manager.install(&source, &options, &DiscardingProgressReporter).await?;

        let template = template_manager.get(&id).unwrap().unwrap();
        let options = RunOptions {
            variant: TemplateVariantInfo::NewApplication,
            name: self.name.clone(),
            output_path: PathBuf::from(&self.name), // TODO: path safe,
            values: Default::default(),
            accept_defaults: false,
        };
        template.run(options).interactive().await
    }

    async fn resolve_selection(&self) -> Result<Option<hub_api::IndexEntry>, dialoguer::Error> {
        let entries = hub_api::index().await.unwrap();
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();

        match matches.len() {
            0 => {
                println!("No templates match your search terms");
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
        index_entry.category() == hub_api::Category::Template
    }
}

fn get_repo_and_id(_: &hub_api::IndexEntry) -> anyhow::Result<(String, String)> {
    // TODO: this
    Ok(("https://github.com/karthik2804/spin-zola".to_owned(), "zola-ssg".to_owned()))
}

struct DiscardingProgressReporter;

impl spin_templates::ProgressReporter for DiscardingProgressReporter {
    fn report(&self, _message: impl AsRef<str>) {
    }
}
