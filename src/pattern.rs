use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    pub pattern: String,
    pub category: String,
    #[serde(default)]
    pub framework: Option<String>,
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub metadata: PatternMetadata,
    pub content: String,
    pub filepath: PathBuf,
}

pub fn patterns_dir() -> Result<PathBuf> {
    let dir = std::env::var("PATTERNS_DIR")
        .map_err(|_| anyhow::anyhow!("PATTERNS_DIR environment variable must be set"))?;
    Ok(PathBuf::from(dir))
}

pub fn load_all() -> Result<Vec<Pattern>> {
    let dir = patterns_dir()?;
    let mut patterns = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "md") {
            if let Some(p) = load_one(&path) {
                patterns.push(p);
            }
        }
    }
    Ok(patterns)
}

fn load_one(path: &std::path::Path) -> Option<Pattern> {
    let text = fs::read_to_string(path).ok()?;
    let rest = text.strip_prefix("---\n")?;
    let (yaml, body) = rest.split_once("\n---\n")?;
    let metadata: PatternMetadata = serde_yaml::from_str(yaml).ok()?;
    Some(Pattern {
        metadata,
        content: body.trim().to_string(),
        filepath: path.to_path_buf(),
    })
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 100 {
        anyhow::bail!("Pattern name must be 1-100 characters");
    }
    if name.chars().any(|c| !c.is_alphanumeric() && c != '-' && c != '_' && c != ' ') {
        anyhow::bail!("Pattern name can only contain alphanumeric, dash, underscore, or space");
    }
    Ok(())
}
