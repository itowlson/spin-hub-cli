use clap::Parser;
use itertools::Itertools;

use crate::hub_api;

#[derive(Parser, Debug)]
#[clap(about = "Search for content on the Hub")]
pub struct SearchCommand {
    terms: Vec<String>,

    #[clap(long, alias = "lang")]
    language: Option<String>,

    #[clap(long, alias = "cat")]
    category: Option<String>,
}

impl SearchCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        let entries = hub_api::index().await?;
        let matches = entries.iter().filter(|e| self.is_match(e)).sorted_by_key(|e| e.title()).collect_vec();
        self.print(&matches);
        Ok(())
    }

    fn print(&self, entries: &[&hub_api::IndexEntry]) {
        if entries.is_empty() {
            println!("No matches");
            return;
        }

        let mut table = comfy_table::Table::new();
        table.load_preset(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);

        let header = vec!["Name", "Description", "Author"];
        table.set_header(header);

        for entry in entries {
            let summary = entry.short_summary();
            let row = vec![entry.title(), summary.as_ref(), entry.author()];
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
        let title = index_entry.title_words();  // TODO: the trouble is now e.g. this picks up 'trigger' for all the templates
        self.terms.iter()
            .map(|t| t.to_lowercase())
            .all(|t| tags.contains(&t) || title.contains(&t))
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
