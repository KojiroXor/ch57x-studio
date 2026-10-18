//! Runtime engine: Event dispatcher, macro runner, window orchestrator, context watcher

pub mod context_actor;
pub mod dispatcher;
pub mod macro_actor;
pub mod window_actor;
pub mod workspace_actor;

pub use context_actor::ContextActor;
pub use dispatcher::{DispatchOutcome, RuntimeDispatcher};
pub use macro_actor::{
    emergency_release_modifiers, ClipboardProvider, InMemoryClipboard, MacroActor, MacroError,
    MacroRunOutcome, OsClipboard,
};
pub use window_actor::{calculate_window_rect, WindowActor, WindowActorConfig};
pub use workspace_actor::{WorkspaceActor, WorkspaceRestoreReport, WorkspaceTargetOutcome};
