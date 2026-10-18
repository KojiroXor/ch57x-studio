//! Deployment journal state machine

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentState {
    Idle,
    ValidatingYaml,
    WritingHardwareFlash,
    ActivatingHostRuntime,
    Success,
    Failed(String),
}
