//! Local trust store for executable hashes and paths

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustStore {
    pub approved_sha256_hashes: HashSet<String>,
}
