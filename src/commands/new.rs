use clap::{Parser};
use itertools::Itertools;
use std::path::PathBuf;
use crate::hub_api;
use anyhow::{Context, Result};
use std::io::{self, Write};


#[derive(Parser, Debug)]
#[clap(about = "Create an application from a template on the Hub")]
pub struct NewCommand {
    #[clap(short = 't')]
    terms: Vec<String>,

    #[clap(name = "name", help = "Name of the application to create from the template")]
    name: Option<String>,
}

impl NewCommand {
    pub async fn run(&self) -> Result<()> {
        
        let Some(index_entry) = self.resolve_selection().await? else {
            return Ok(());
        };
    
        println!("Template {} by {}", index_entry.title(), index_entry.author());
        println!("{}", index_entry.summary());
    
        let app_name = if let Some(ref name) = self.name {
            name.clone()
        } else {
            print!("Enter a name for your new application: ");
            io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        };
    
        let (repo, id) = get_repo_and_id(&index_entry)?;
    
        self.run_template(repo, id, app_name).await
    }
    
    async fn run_template(&self, repo: String, id: String, app_name: String) -> Result<()> {
        use spin_templates::*;
        
        let tempdir = tempfile::tempdir().unwrap();
        let manager = spin_templates::TemplateManager::in_dir(tempdir.path());
    
        let source = TemplateSource::try_from_git(&repo, &None, &crate::spin::version())?;
        let options = InstallOptions::default();
        manager.install(&source, &options, &DiscardingProgressReporter).await?;
    
        let template = match manager.get(&id)? {
            Some(template) => template,
            None => return Err(anyhow::anyhow!("Template not found in the repository.")),
        };
            
        let options = RunOptions {
            variant: TemplateVariantInfo::NewApplication,
            name: app_name.clone(),
            output_path: PathBuf::from(&app_name),
            values: Default::default(),
            accept_defaults: false,
        };

        template.run(options).interactive().await
    }

    async fn resolve_selection(&self) -> Result<Option<hub_api::IndexEntry>> {
        let entries = hub_api::index().await.unwrap();
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();

        if matches.is_empty() {
            println!("No templates match your search terms");
            return Ok(None);
        }

        let selection = dialoguer::Select::new()
            .with_prompt("Select a template:")
            .items(&matches.iter().map(|entry| format!("{} - {}", entry.title(), entry.summary())).collect_vec())
            .interact_opt()?;

        if let Some(idx) = selection {
            Ok(Some(matches[idx].clone()))
        } else {
            Ok(None)
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

fn get_repo_and_id(index_entry: &hub_api::IndexEntry) -> Result<(String, String)> {
    let repo_url = index_entry.repo_url();
    let template_id = index_entry.template_id(); 

    Ok((repo_url.to_string(), template_id.to_string()))
}

struct DiscardingProgressReporter;

impl spin_templates::ProgressReporter for DiscardingProgressReporter {
    fn report(&self, _message: impl AsRef<str>) {
    }
}
