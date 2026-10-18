//! UI state

use ch57x_domain::hardware::LayerIndex;

pub struct UiState {
    pub editing_layer: LayerIndex,
    pub selected_slot_index: Option<usize>,
    pub is_dirty: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            editing_layer: LayerIndex(0),
            selected_slot_index: None,
            is_dirty: false,
        }
    }
}
