use std::path::{PathBuf, Path};
use std::fs;
use clap::Parser;
use itertools::Itertools;
use fs_extra::dir::{copy, CopyOptions};
use crate::{hub_api, git};

#[derive(Parser, Debug)]
#[clap(about = "Get a sample from a Hub")]
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

        let (repo, subdirectory) = get_repo_and_directory(&index_entry)?;


        let repo_name = repo.split('/').last().unwrap().replace(".git", "");
        let clone_dir = PathBuf::from(&repo_name);

        let cloned = clone_dir.exists();
        if !cloned {

            println!("Cloning repository: {:?}", repo);
            git::clone_decoupled(&repo).await?;
        }

        if !subdirectory.is_empty() {
            let sample_dir = clone_dir.join(&subdirectory);

            println!("Checking sub-directory path: {:?}", sample_dir);


            if !self.is_safe_path(&sample_dir)? {
                return Err(anyhow::anyhow!("Unsafe directory path detected."));
            }

            if !sample_dir.exists() {
                return Err(anyhow::anyhow!("Specified sample directory does not exist: {:?}", sample_dir));
            }


            let mut options = CopyOptions::new();
            options.skip_exist = true; 


            let new_dir = PathBuf::from(".").join(sample_dir.file_name().unwrap());
            println!("Copying sub-directory to current directory: {:?}", new_dir);
            copy(&sample_dir, ".", &options)?;


            if clone_dir.exists() {
                println!("Deleting cloned repository at: {:?}", clone_dir);
                match fs::remove_dir_all(&clone_dir) {
                    Ok(_) => println!("Successfully deleted cloned repository."),
                    Err(e) => eprintln!("Failed to delete cloned repository: {:?}", e),
                }
            }


            std::env::set_current_dir(&new_dir)?;
            println!("Changed working directory to: {:?}", new_dir);


            let verb = if self.deploy {
                "deploy"
            } else {
                "up"
            };

            let mut child = match crate::spin::bin() {
                Ok(command) => command,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e); 
                }
            }
            .arg(verb)
            .args(["--build", "-f"])
            .arg("spin.toml")
            .spawn()?;

            _ = child.wait().await;
        } else {
            std::env::set_current_dir(&clone_dir)?;
            println!("Running command in cloned repository root: {:?}", clone_dir);

            let verb = if self.deploy {
                "deploy"
            } else {
                "up"
            };

            let mut child = match crate::spin::bin() {
                Ok(command) => command,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e); 
                }
            }
            .arg(verb)
            .args(["--build", "-f"])
            .arg("spin.toml")
            .spawn()?;

            _ = child.wait().await;


            if clone_dir.exists() {
                println!("Deleting cloned repository at: {:?}", clone_dir);
                match fs::remove_dir_all(&clone_dir) {
                    Ok(_) => println!("Successfully deleted cloned repository."),
                    Err(e) => eprintln!("Failed to delete cloned repository: {:?}", e),
                }
            }
        }

        Ok(())
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
        index_entry.category() == hub_api::Category::Sample
    }

    fn is_safe_path(&self, path: &Path) -> anyhow::Result<bool> {
        let path_str = path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid path"))?;
        if path_str.contains("..") || path_str.contains(";") || path_str.contains("$") {
            return Ok(false);
        }
        Ok(true)
    }
}

fn get_repo_and_directory(index_entry: &hub_api::IndexEntry) -> anyhow::Result<(String, String)> {
    // Extract the repository URL
    let repo_url = index_entry.repo_url().to_owned();
    
    // Get the subdirectory directly
    let directory = index_entry.subdirectory();
    
    Ok((repo_url, directory.to_owned()))
}