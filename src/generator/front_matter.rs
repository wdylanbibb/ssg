use serde::{Deserialize, Serialize};

use super::error::FrontMatterError;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct FrontMatter {
    pub(super) title: String,
    #[serde(default = "default_template")]
    pub(super) template: String,
    #[serde(default)]
    pub(super) description: String,
}

pub(super) fn parse_document(document: &str) -> Result<(FrontMatter, &str), FrontMatterError> {
    let (yaml, markdown) = extract_front_matter(document)?;
    let front_matter = serde_yaml::from_str(yaml)?;

    Ok((front_matter, markdown))
}

fn extract_front_matter(document: &str) -> Result<(&str, &str), FrontMatterError> {
    let document = document.strip_prefix('\u{feff}').unwrap_or(document);

    let remainder = document
        .strip_prefix("---\n")
        .ok_or(FrontMatterError::MissingOpeningDelimeter)?;

    let mut offset = 0;

    for line in remainder.split_inclusive('\n') {
        let line_without_ending = line.trim_end_matches(['\n']);

        if line_without_ending == "---" {
            let yaml = &remainder[..offset];
            let markdown = &remainder[offset + line.len()..];

            return Ok((yaml, markdown));
        }

        offset += line.len();
    }

    Err(FrontMatterError::MissingClosingDelimeter)
}

fn default_template() -> String {
    "page.html".to_owned()
}
