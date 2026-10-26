//! `atlas-core`: the knowledge graph behind the learning layer.
//!
//! Public surface is intentionally small: [`Atlas`] for operations on a vault,
//! and the types in [`model`] for what it stores.

mod atlas;
mod error;
pub mod model;

pub use atlas::{Atlas, ConceptContext};
pub use error::{Error, Result};
pub use model::{Concept, Edge, EdgeKind, Evidence, Id, Source};
