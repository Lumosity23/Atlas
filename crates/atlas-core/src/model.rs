//! Domain types. Nothing here performs IO.
//!
//! The model deliberately stores no "mastery score". What you know is derived
//! from [`Evidence`] that you demonstrated (`verified == true`), so every claim
//! of knowledge points back to where it was shown.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub type Id = String;

/// Unix time in seconds.
pub type Timestamp = u64;

pub fn new_id() -> Id {
    uuid::Uuid::new_v4().to_string()
}

pub fn now() -> Timestamp {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A unit of understanding: "Dérivée d'une fonction", "Théorème de Bayes", etc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    pub id: Id,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// `from` cannot be understood without `to`. The only kind that must stay acyclic.
    Prerequisite,
    /// Same territory, useful to see together.
    Related,
    /// Often confused with, or a counterpart of, the other concept.
    Contrast,
    /// `from` is a worked instance of `to`.
    Example,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    pub from: Id,
    pub to: Id,
    pub kind: EdgeKind,
}

/// Where a piece of evidence came from. Later this will also cover
/// photographed paper pages and drawings from the tablet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Source {
    Note { page: String },
    Exercise { reference: String },
    Conversation { session: String },
    Drawing { asset: String },
    Manual,
}

/// A trace that a concept was touched, explained or demonstrated.
///
/// `verified` is the only thing that makes a concept count as known:
/// it means you produced the understanding yourself (an answer, an explanation,
/// a solved exercise). Reading or writing a note does not verify anything.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Id,
    pub concepts: Vec<Id>,
    pub source: Source,
    pub body: String,
    pub verified: bool,
    pub at: Timestamp,
}
