use clap::{Parser};

mod hub_api;

#[tokio::main]
async fn main() {
    HubCommand::parse().run().await;
}

#[derive(Parser)]
#[clap(about = "Commands for using content from the Spin Up Hub")]
enum HubCommand {
    Search(SearchCommand),
}

impl HubCommand {
    async fn run(&self) {
        match self {
            Self::Search(cmd) => cmd.run().await,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(about = "Search for content on the Hub")]
struct SearchCommand {
    terms: Vec<String>,

    #[clap(long, alias = "lang")]
    language: Option<String>,

    #[clap(long, alias = "cat")]
    category: Option<String>,
}

use itertools::Itertools;

impl SearchCommand {
    async fn run(&self) {
        let entries = hub_api::index().await.unwrap();
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();
        self.print(&matches);
    }

    fn print(&self, entries: &[&hub_api::IndexEntry]) {
        if entries.is_empty() {
            println!("No matches");
        }

        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);

        let header = vec!["Name", "Description", "Author"];
        table.set_header(header);

        for entry in entries {
            let summary = entry.short_summary();
            let row = vec![entry.title(), summary.as_str(), entry.author()];
            table.add_row(row);
        }

        println!("{table}");
    }

    fn is_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        self.is_terms_match(index_entry) &&
            self.is_lang_match(index_entry) &&
            self.is_category_match(index_entry)
    }

    fn is_terms_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        let tags = index_entry.tags();
        self.terms.iter().all(|t| tags.contains(t))
    }

    fn is_lang_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        match &self.language {
            None => true,
            Some(lang) => index_entry.language().is_match(lang),
        }
    }

    fn is_category_match(&self, index_entry: &hub_api::IndexEntry) -> bool {
        match &self.category {
            None => true,
            Some(cat) => index_entry.category() == hub_api::Category::parse(cat),
        }
    }
}
