//! `atlas`: a small CLI over the knowledge vault.
//!
//! Every read command accepts `--json`, so an agent can call it and parse the
//! output without scraping text.

use std::path::PathBuf;
use std::process::ExitCode;

use atlas_core::{Atlas, EdgeKind, Source};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "atlas", version, about = "Personal knowledge graph with learning evidence")]
struct Cli {
    /// Vault directory. Defaults to $ATLAS_VAULT, then ./atlas-vault.
    #[arg(long, env = "ATLAS_VAULT", default_value = "atlas-vault", global = true)]
    vault: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Manage concepts.
    #[command(subcommand)]
    Concept(ConceptCmd),
    /// Link two concepts.
    Link {
        from: String,
        to: String,
        #[arg(long, value_enum, default_value_t = KindArg::Prerequisite)]
        kind: KindArg,
    },
    /// Record evidence that concepts were touched or demonstrated.
    Evidence {
        /// Concept ids. Repeat for several.
        #[arg(long = "concept", required = true)]
        concepts: Vec<String>,
        /// Where it came from.
        #[arg(long, default_value = "manual")]
        source: String,
        /// Mark as demonstrated by the learner. Only verified evidence counts as known.
        #[arg(long)]
        verified: bool,
        /// Free text: the answer, the explanation, the note.
        body: String,
    },
    /// List concepts the learner has demonstrated.
    Known {
        #[arg(long)]
        json: bool,
    },
    /// Study order to reach a goal, skipping what is already known.
    Plan {
        goal: String,
        #[arg(long)]
        json: bool,
    },
    /// Full context for one concept: prerequisites, links, evidence.
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ConceptCmd {
    /// Create a concept.
    Add {
        title: String,
        #[arg(long, default_value = "")]
        summary: String,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    /// List all concepts.
    List {
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum KindArg {
    Prerequisite,
    Related,
    Contrast,
    Example,
}

impl From<KindArg> for EdgeKind {
    fn from(kind: KindArg) -> Self {
        match kind {
            KindArg::Prerequisite => EdgeKind::Prerequisite,
            KindArg::Related => EdgeKind::Related,
            KindArg::Contrast => EdgeKind::Contrast,
            KindArg::Example => EdgeKind::Example,
        }
    }
}

/// Accepts `manual`, `note:<page>`, `exercise:<ref>`, `conversation:<session>`, `drawing:<asset>`.
fn parse_source(raw: &str) -> Source {
    match raw.split_once(':') {
        Some(("note", page)) => Source::Note { page: page.to_owned() },
        Some(("exercise", reference)) => Source::Exercise { reference: reference.to_owned() },
        Some(("conversation", session)) => Source::Conversation { session: session.to_owned() },
        Some(("drawing", asset)) => Source::Drawing { asset: asset.to_owned() },
        _ => Source::Manual,
    }
}

fn emit<T: serde::Serialize>(json: bool, value: &T, text: impl FnOnce() -> String) -> Result<(), String> {
    if json {
        println!("{}", serde_json::to_string_pretty(value).map_err(|e| e.to_string())?);
    } else {
        println!("{}", text());
    }
    Ok(())
}

fn run(cli: Cli) -> Result<(), String> {
    let mut atlas = Atlas::open(&cli.vault).map_err(|e| e.to_string())?;
    let map_err = |e: atlas_core::Error| e.to_string();

    match cli.command {
        Command::Concept(ConceptCmd::Add { title, summary, tags }) => {
            let id = atlas.add_concept(title, summary, tags).map_err(map_err)?;
            println!("{id}");
        }
        Command::Concept(ConceptCmd::List { json }) => {
            let concepts: Vec<_> = atlas.concepts().collect();
            let known = atlas.known();
            emit(json, &concepts, || {
                concepts
                    .iter()
                    .map(|c| {
                        let mark = if known.contains(&c.id) { "✓" } else { " " };
                        format!("[{mark}] {}  {}", c.id, c.title)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        }
        Command::Link { from, to, kind } => {
            atlas.link(&from, &to, kind.into()).map_err(map_err)?;
            println!("linked {from} -> {to}");
        }
        Command::Evidence { concepts, source, verified, body } => {
            let id = atlas
                .add_evidence(concepts, parse_source(&source), body, verified)
                .map_err(map_err)?;
            println!("{id}");
        }
        Command::Known { json } => {
            let known: Vec<_> = atlas.known().into_iter().collect();
            emit(json, &known, || {
                known
                    .iter()
                    .filter_map(|id| atlas.concept(id).ok())
                    .map(|c| format!("✓ {}  {}", c.id, c.title))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        }
        Command::Plan { goal, json } => {
            let order = atlas.plan(&goal).map_err(map_err)?;
            emit(json, &order, || {
                order
                    .iter()
                    .enumerate()
                    .filter_map(|(i, id)| atlas.concept(id).ok().map(|c| format!("{:>2}. {}  {}", i + 1, c.id, c.title)))
                    .collect::<Vec<_>>()
                    .join("\n")
            })?;
        }
        Command::Show { id, json } => {
            let ctx = atlas.context(&id).map_err(map_err)?;
            emit(json, &ctx, || {
                let mut out = format!(
                    "{}  {}\n{}\nknown: {}\n",
                    ctx.concept.id, ctx.concept.title, ctx.concept.summary, ctx.known
                );
                out.push_str("prerequisites:\n");
                for p in &ctx.prerequisites {
                    let title = atlas.concept(p).map(|c| c.title.as_str()).unwrap_or("?");
                    out.push_str(&format!("  - {p}  {title}\n"));
                }
                out.push_str(&format!("evidence: {}\n", ctx.evidence.len()));
                for e in &ctx.evidence {
                    out.push_str(&format!("  - [{}] {}\n", if e.verified { "verified" } else { "touched" }, e.body));
                }
                out
            })?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
