use crate::actions::{ActionDefinition, ActionId, SlotBinding};
use crate::hardware::{DeviceDescriptor, SlotId, TOTAL_SLOTS};
use crate::triggers::PhantomTrigger;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlotConfig {
    pub slot: SlotId,
    pub label: String,
    pub color_hex: Option<String>,
    pub binding: SlotBinding,
}

impl SlotConfig {
    pub fn default_for_slot(slot: SlotId) -> Self {
        Self {
            slot,
            label: format!("Slot {}", slot.global_index() + 1),
            color_hex: None,
            binding: SlotBinding::Disabled,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub id: Uuid,
    pub name: String,
    pub version: u32,
    pub device: DeviceDescriptor,
    pub slots: HashMap<usize, SlotConfig>, // Keyed by global_index (0..53)
    pub actions: HashMap<ActionId, ActionDefinition>,
    pub trigger_allocation: HashMap<usize, PhantomTrigger>, // Keyed by global_index (0..53)
}

impl Default for Profile {
    fn default() -> Self {
        let mut slots = HashMap::with_capacity(TOTAL_SLOTS);
        for slot in SlotId::all_slots() {
            let idx = slot.global_index();
            slots.insert(idx, SlotConfig::default_for_slot(slot));
        }

        Self {
            id: Uuid::new_v4(),
            name: "Default Profile".into(),
            version: 1,
            device: DeviceDescriptor::default(),
            slots,
            actions: HashMap::new(),
            trigger_allocation: HashMap::new(),
        }
    }
}
