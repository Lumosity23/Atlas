//! The knowledge graph and its file-backed storage.
//!
//! Layout on disk (plain, inspectable files):
//! - `concepts.json`   — all concepts
//! - `edges.json`      — all links between concepts
//! - `evidence.jsonl`  — append-only log of evidence, one entry per line

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{self, Concept, Edge, EdgeKind, Evidence, Id, Source};

const CONCEPTS_FILE: &str = "concepts.json";
const EDGES_FILE: &str = "edges.json";
const EVIDENCE_FILE: &str = "evidence.jsonl";

/// Everything an agent needs to reason about one concept.
#[derive(Debug, Clone, Serialize)]
pub struct ConceptContext {
    pub concept: Concept,
    /// Whether the learner has demonstrated this concept.
    pub known: bool,
    /// Transitive prerequisites, nearest first.
    pub prerequisites: Vec<Id>,
    /// Concepts linked by `related`, `contrast` or `example` in either direction.
    pub linked: Vec<(Id, EdgeKind)>,
    pub evidence: Vec<Evidence>,
}

pub struct Atlas {
    root: PathBuf,
    concepts: BTreeMap<Id, Concept>,
    edges: Vec<Edge>,
    evidence: Vec<Evidence>,
}

impl Atlas {
    /// Open a vault directory, creating it (and empty files) if needed.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;

        let concepts: Vec<Concept> = read_json_or_default(&root.join(CONCEPTS_FILE))?;
        let edges: Vec<Edge> = read_json_or_default(&root.join(EDGES_FILE))?;
        let evidence = read_jsonl(&root.join(EVIDENCE_FILE))?;

        Ok(Self {
            root,
            concepts: concepts.into_iter().map(|c| (c.id.clone(), c)).collect(),
            edges,
            evidence,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    // ----- concepts ---------------------------------------------------------

    pub fn add_concept(
        &mut self,
        title: impl Into<String>,
        summary: impl Into<String>,
        tags: Vec<String>,
    ) -> Result<Id> {
        let concept = Concept {
            id: model::new_id(),
            title: title.into(),
            summary: summary.into(),
            tags,
            created_at: model::now(),
        };
        let id = concept.id.clone();
        self.concepts.insert(id.clone(), concept);
        self.save_concepts()?;
        Ok(id)
    }

    pub fn concept(&self, id: &str) -> Result<&Concept> {
        self.concepts
            .get(id)
            .ok_or_else(|| Error::UnknownConcept(id.to_owned()))
    }

    pub fn concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.values()
    }

    // ----- links ------------------------------------------------------------

    /// Add a link. A prerequisite link is refused if it would create a cycle.
    pub fn link(&mut self, from: &str, to: &str, kind: EdgeKind) -> Result<()> {
        self.concept(from)?;
        self.concept(to)?;

        if from == to {
            return Err(Error::SelfLink(from.to_owned()));
        }
        if self.edges.iter().any(|e| e.from == from && e.to == to && e.kind == kind) {
            return Ok(());
        }
        if kind == EdgeKind::Prerequisite && self.prerequisite_closure(to).contains(from) {
            return Err(Error::PrerequisiteCycle {
                from: from.to_owned(),
                to: to.to_owned(),
            });
        }

        self.edges.push(Edge {
            from: from.to_owned(),
            to: to.to_owned(),
            kind,
        });
        self.save_edges()
    }

    // ----- evidence ---------------------------------------------------------

    pub fn add_evidence(
        &mut self,
        concepts: Vec<Id>,
        source: Source,
        body: impl Into<String>,
        verified: bool,
    ) -> Result<Id> {
        for id in &concepts {
            self.concept(id)?;
        }
        let entry = Evidence {
            id: model::new_id(),
            concepts,
            source,
            body: body.into(),
            verified,
            at: model::now(),
        };
        let id = entry.id.clone();
        append_jsonl(&self.root.join(EVIDENCE_FILE), &entry)?;
        self.evidence.push(entry);
        Ok(id)
    }

    pub fn evidence_for(&self, concept: &str) -> Vec<&Evidence> {
        self.evidence
            .iter()
            .filter(|e| e.concepts.iter().any(|c| c == concept))
            .collect()
    }

    // ----- derived views ----------------------------------------------------

    /// Concepts with at least one verified piece of evidence. Derived, never stored.
    pub fn known(&self) -> BTreeSet<Id> {
        self.evidence
            .iter()
            .filter(|e| e.verified)
            .flat_map(|e| e.concepts.iter().cloned())
            .collect()
    }

    /// All concepts `id` transitively depends on, excluding `id` itself.
    pub fn prerequisite_closure(&self, id: &str) -> BTreeSet<Id> {
        let mut seen = BTreeSet::new();
        let mut queue: VecDeque<&str> = VecDeque::from([id]);
        while let Some(current) = queue.pop_front() {
            for edge in self.edges.iter().filter(|e| {
                e.kind == EdgeKind::Prerequisite && e.from == current
            }) {
                if seen.insert(edge.to.clone()) {
                    queue.push_back(&edge.to);
                }
            }
        }
        seen
    }

    /// Study order to reach `goal`: unknown prerequisites first, goal last.
    ///
    /// Concepts already verified are skipped, and so is anything beneath them,
    /// since knowing a concept means its foundations were in place.
    pub fn plan(&self, goal: &str) -> Result<Vec<Id>> {
        self.concept(goal)?;
        let known = self.known();
        let mut order = Vec::new();
        let mut visited = BTreeSet::new();
        self.visit_for_plan(goal, &known, &mut visited, &mut order);
        Ok(order)
    }

    fn visit_for_plan(
        &self,
        id: &str,
        known: &BTreeSet<Id>,
        visited: &mut BTreeSet<Id>,
        order: &mut Vec<Id>,
    ) {
        if known.contains(id) || !visited.insert(id.to_owned()) {
            return;
        }
        for edge in self
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Prerequisite && e.from == id)
        {
            self.visit_for_plan(&edge.to, known, visited, order);
        }
        order.push(id.to_owned());
    }

    /// Everything an agent needs to talk about one concept.
    pub fn context(&self, id: &str) -> Result<ConceptContext> {
        let concept = self.concept(id)?.clone();
        let known = self.known().contains(id);

        let mut prerequisites: Vec<Id> = self.prerequisite_closure(id).into_iter().collect();
        prerequisites.sort_by_key(|p| self.depth_from(id, p));

        let linked = self
            .edges
            .iter()
            .filter(|e| e.kind != EdgeKind::Prerequisite)
            .filter_map(|e| {
                if e.from == id {
                    Some((e.to.clone(), e.kind))
                } else if e.to == id {
                    Some((e.from.clone(), e.kind))
                } else {
                    None
                }
            })
            .collect();

        Ok(ConceptContext {
            concept,
            known,
            prerequisites,
            linked,
            evidence: self.evidence_for(id).into_iter().cloned().collect(),
        })
    }

    /// Shortest number of prerequisite hops from `from` to `to`, used to sort.
    fn depth_from(&self, from: &str, to: &str) -> usize {
        let mut frontier: Vec<&str> = vec![from];
        let mut seen: BTreeSet<&str> = BTreeSet::from([from]);
        for depth in 1.. {
            let mut next = Vec::new();
            for node in &frontier {
                for edge in self
                    .edges
                    .iter()
                    .filter(|e| e.kind == EdgeKind::Prerequisite && e.from == *node)
                {
                    if edge.to == to {
                        return depth;
                    }
                    if seen.insert(&edge.to) {
                        next.push(edge.to.as_str());
                    }
                }
            }
            if next.is_empty() {
                return usize::MAX;
            }
            frontier = next;
        }
        usize::MAX
    }

    // ----- persistence ------------------------------------------------------

    fn save_concepts(&self) -> Result<()> {
        let list: Vec<&Concept> = self.concepts.values().collect();
        write_json_atomic(&self.root.join(CONCEPTS_FILE), &list)
    }

    fn save_edges(&self) -> Result<()> {
        write_json_atomic(&self.root.join(EDGES_FILE), &self.edges)
    }
}

fn read_json_or_default<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_str(&text)?)
}

fn read_jsonl<T>(path: &Path) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.exists() {
        return Ok(Vec::new());
    }
    let reader = BufReader::new(File::open(path)?);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if !line.trim().is_empty() {
            out.push(serde_json::from_str(&line)?);
        }
    }
    Ok(out)
}

fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(value)?)?;
    fs::rename(tmp, path)?;
    Ok(())
}
