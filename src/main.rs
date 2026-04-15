mod browse;
mod output;
mod pattern;
mod setup;

use anyhow::Result;
use clap::{Parser, Subcommand};
use output::Output;
use pattern::{load_all, validate_name};

#[derive(Parser)]
#[command(name = "grimoire", version, about = "Pattern library CLI — agent-ready")]
struct Cli {
    /// Output as JSON (errors also become JSON on stderr)
    #[arg(long, global = true)]
    json: bool,

    /// JMESPath query to filter JSON output
    #[arg(long, global = true, value_name = "EXPR")]
    query: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all available patterns
    List,

    /// Search patterns by text, category, framework, or tag
    Search {
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        framework: Option<String>,
        #[arg(long)]
        tag: Option<String>,
    },

    /// Get a specific pattern by name (full content)
    Get { name: String },

    /// Create a new pattern
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        category: String,
        #[arg(long)]
        framework: String,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        projects: Vec<String>,
        #[arg(long)]
        content: String,
        /// Preview without writing to disk
        #[arg(long)]
        dry_run: bool,
    },

    /// Browse patterns interactively (TUI)
    Browse,

    /// Install SKILL.md for Kiro
    SetupKiro {
        #[arg(long)]
        dry_run: bool,
    },
    /// Install SKILL.md for OpenCode
    SetupOc {
        #[arg(long)]
        dry_run: bool,
    },
    /// Install SKILL.md for Codex
    SetupCodex {
        #[arg(long)]
        dry_run: bool,
    },
    /// Install SKILL.md for Claude
    SetupClaude {
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let out = Output::new(cli.json, cli.query.as_deref());
    if let Err(e) = run(cli.command, &out) {
        out.error(&e.to_string());
        std::process::exit(1);
    }
}

fn run(cmd: Commands, out: &Output) -> Result<()> {
    match cmd {
        Commands::List => {
            let patterns = load_all()?;
            if out.is_json() {
                let items: Vec<_> = patterns.iter().map(|p| serde_json::json!({
                    "pattern": p.metadata.pattern,
                    "category": p.metadata.category,
                    "framework": p.metadata.framework,
                    "tags": p.metadata.tags,
                })).collect();
                out.print_json(&serde_json::Value::Array(items))?;
            } else {
                for p in &patterns {
                    println!("{} ({})", p.metadata.pattern, p.metadata.category);
                }
            }
        }

        Commands::Search { text, category, framework, tag } => {
            let patterns = load_all()?;
            let results: Vec<_> = patterns.iter().filter(|p| {
                category.as_ref().is_none_or(|c| &p.metadata.category == c)
                    && framework.as_ref().is_none_or(|f| p.metadata.framework.as_deref() == Some(f.as_str()))
                    && tag.as_ref().is_none_or(|t| p.metadata.tags.contains(t))
                    && text.as_ref().is_none_or(|q| {
                        format!("{} {}", p.metadata.pattern, p.content)
                            .to_lowercase()
                            .contains(&q.to_lowercase())
                    })
            }).collect();

            if out.is_json() {
                let items: Vec<_> = results.iter().map(|p| serde_json::json!({
                    "pattern": p.metadata.pattern,
                    "category": p.metadata.category,
                    "framework": p.metadata.framework,
                    "tags": p.metadata.tags,
                    "preview": &p.content[..200.min(p.content.len())],
                })).collect();
                out.print_json(&serde_json::Value::Array(items))?;
            } else {
                for p in &results {
                    println!("{} ({})", p.metadata.pattern, p.metadata.category);
                }
            }
        }

        Commands::Get { name } => {
            let patterns = load_all()?;
            let p = patterns.iter()
                .find(|p| p.metadata.pattern == name)
                .ok_or_else(|| anyhow::anyhow!("Pattern '{}' not found", name))?;

            if out.is_json() {
                out.print_json(&serde_json::json!({
                    "pattern": p.metadata.pattern,
                    "category": p.metadata.category,
                    "framework": p.metadata.framework,
                    "tags": p.metadata.tags,
                    "projects": p.metadata.projects,
                    "content": p.content,
                }))?;
            } else {
                println!("{}", p.content);
            }
        }

        Commands::Create { name, category, framework, tags, projects, content, dry_run } => {
            validate_name(&name)?;

            let projects_str = if projects.is_empty() { String::new() }
                else { format!("projects: [{}]\n", projects.join(", ")) };
            let tags_str = if tags.is_empty() { String::new() }
                else { format!("tags: [{}]\n", tags.join(", ")) };

            let file_content = format!(
                "---\npattern: {name}\ncategory: {category}\nframework: {framework}\n{projects_str}{tags_str}---\n\n{content}\n"
            );
            let filename = name.replace(' ', "-").to_lowercase();
            let dir = pattern::patterns_dir()?;
            let path = dir.join(format!("{filename}.md"));

            if dry_run {
                if out.is_json() {
                    out.print_json(&serde_json::json!({ "dry_run": true, "path": path, "content": file_content }))?;
                } else {
                    println!("Would write: {}\n{}", path.display(), file_content);
                }
            } else {
                std::fs::write(&path, &file_content)?;
                if out.is_json() {
                    out.print_json(&serde_json::json!({ "created": path }))?;
                } else {
                    println!("Created: {}", path.display());
                }
            }
        }

        Commands::Browse => browse::run()?,

        Commands::SetupKiro { dry_run } => {
            let home = std::env::var("HOME")?;
            setup::install(&format!("{home}/.kiro/skills/coding-patterns/SKILL.md"), dry_run, out)?;
        }
        Commands::SetupOc { dry_run } => {
            let home = std::env::var("HOME")?;
            setup::install(&format!("{home}/.config/opencode/skills/coding-patterns/SKILL.md"), dry_run, out)?;
        }
        Commands::SetupCodex { dry_run } => {
            let home = std::env::var("HOME")?;
            setup::install(&format!("{home}/.codex/skills/coding-patterns/SKILL.md"), dry_run, out)?;
        }
        Commands::SetupClaude { dry_run } => {
            let home = std::env::var("HOME")?;
            setup::install(&format!("{home}/.claude/skills/coding-patterns/SKILL.md"), dry_run, out)?;
        }
    }
    Ok(())
}
