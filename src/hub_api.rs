use itertools::Itertools;

const DEV_SITE_BASE: &'static str = "https://developer.fermyon.com";

fn index_url() -> url::Url {
    url::Url::parse(DEV_SITE_BASE)
        .expect("Base URL was malformed")
        .join("api/hub/get_list")
        .expect("Index URL was malformed")
}

pub async fn index() -> Result<Vec<IndexEntry>, Error> {
    let response = reqwest::get(index_url()).await?;
    if !response.status().is_success() {
        return Err(Error::Response(response.status()));
    }
    let body = response.bytes().await?;
    Ok(serde_json::from_slice(&body)?)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Request(#[from] reqwest::Error),
    #[error("Unsuccessful response from API: {0}")]
    Response(reqwest::StatusCode),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct IndexEntry {
    title: String,
    summary: String,
    category: String,
    language: String,
    author: String,
    tags: Vec<String>,
    #[allow(dead_code)]
    path: String,
}

const SHORT_SUMMARY_LEN: usize = 60;

impl IndexEntry {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn summary(&self) -> &str {
        &self.summary
    }

    pub fn short_summary(&self) -> String {
        if self.summary.len() < SHORT_SUMMARY_LEN {
            self.summary.clone()
        } else {
            let suffix = "...";
            let max_len = SHORT_SUMMARY_LEN - suffix.len();
            let truncated = truncate_to_word_boundary(&self.summary, max_len, 5);
            format!("{truncated}{suffix}")
        }
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn language(&self) -> Language {
        match self.language.to_lowercase().as_str() {
            "rust" => Language::Rust,
            "js/ts" | "javascript" | "typescript" => Language::JavaScript,
            "python" => Language::Python,
            "go" | "tinygo" => Language::Go,
            _ => Language::Other(self.language.clone()),
        }
    }

    pub fn category(&self) -> Category {
        Category::parse(&self.category)
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.to_lowercase()).collect_vec()
    }

    pub fn title_words(&self) -> Vec<String> {
        self.title.split_whitespace().map(|t| t.to_lowercase()).collect_vec()
    }
}

#[derive(Debug, PartialEq)]
pub enum Language {
    #[allow(dead_code)]
    Neutral,
    Rust,
    JavaScript,
    Python,
    Go,
    Other(String),
}

impl Language {
    pub fn is_match(&self, lang: &str) -> bool {
        let lang = lang.to_lowercase();
        match self {
            Self::Neutral => true, // TODO: or... false?
            Self::Rust => lang == "rust" || lang == "rs",
            Self::JavaScript => lang == "javascript" || lang == "js" || lang == "typescript" || lang == "ts",
            Self::Python => lang == "python" || lang == "python3" || lang == "py",
            Self::Go => lang == "go" || lang == "tinygo" || lang == "golang",
            Self::Other(name) => lang == name.as_str(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Category {
    // Component,
    Library,
    Plugin,
    Template,
    Sample,
    Other(String),
}

impl Category {
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "library" => Category::Library,
            "plugin" => Category::Plugin,
            "sample" => Category::Sample,
            "template" => Category::Template,
            _ => Category::Other(value.to_string()),
        }
    }
}

fn truncate_to_word_boundary(source: &str, max_len: usize, min_len: usize) -> &str {
    if source.len() <= max_len {
        return source;
    }

    let mut index = max_len - 1;
    loop {
        let ch = source.chars().nth(index).unwrap();
        if ch.is_whitespace() {
            return source[..index].trim();
        }
        index = index - 1;
        if index < min_len {
            break;
        }
    }

    &source[..max_len]
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn truncation() {
        assert_eq!("Hello world", truncate_to_word_boundary("Hello world bibblybobbly", 15, 2));
        assert_eq!("Hello world", truncate_to_word_boundary("Hello world", 15, 2));
        assert_eq!("Hello", truncate_to_word_boundary("Hello world", 7, 2));
        assert_eq!("Hello", truncate_to_word_boundary("Hello world", 5, 2));
        assert_eq!("Hell", truncate_to_word_boundary("Hello world", 4, 2));
    }
}
