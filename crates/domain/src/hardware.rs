use serde::{Deserialize, Serialize};

pub const LAYER_COUNT: usize = 3;
pub const ROW_COUNT: usize = 3;
pub const COL_COUNT: usize = 4;
pub const BUTTONS_PER_LAYER: usize = ROW_COUNT * COL_COUNT; // 12
pub const ENCODERS_PER_LAYER: usize = 2;
pub const EVENTS_PER_ENCODER: usize = 3; // CCW, Press, CW
pub const SLOTS_PER_LAYER: usize = BUTTONS_PER_LAYER + (ENCODERS_PER_LAYER * EVENTS_PER_ENCODER); // 18
pub const TOTAL_SLOTS: usize = LAYER_COUNT * SLOTS_PER_LAYER; // 54

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayerIndex(pub u8); // 0..=2 (mostrato in UI come Layer 1..=3)

impl LayerIndex {
    pub fn new(index: u8) -> Option<Self> {
        if (index as usize) < LAYER_COUNT {
            Some(Self(index))
        } else {
            None
        }
    }

    pub fn display_number(&self) -> u8 {
        self.0 + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ControlId {
    Button { row: u8, col: u8 },                // row: 0..=2, col: 0..=3
    Encoder { index: u8, event: EncoderEvent }, // index: 0 (Top), 1 (Bottom)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EncoderEvent {
    Ccw,
    Press,
    Cw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SlotId {
    pub layer: LayerIndex,
    pub control: ControlId,
}

impl SlotId {
    pub fn new_button(layer: u8, row: u8, col: u8) -> Option<Self> {
        if (layer as usize) < LAYER_COUNT
            && (row as usize) < ROW_COUNT
            && (col as usize) < COL_COUNT
        {
            Some(Self {
                layer: LayerIndex(layer),
                control: ControlId::Button { row, col },
            })
        } else {
            None
        }
    }

    pub fn new_encoder(layer: u8, index: u8, event: EncoderEvent) -> Option<Self> {
        if (layer as usize) < LAYER_COUNT && (index as usize) < ENCODERS_PER_LAYER {
            Some(Self {
                layer: LayerIndex(layer),
                control: ControlId::Encoder { index, event },
            })
        } else {
            None
        }
    }

    /// Calcola l'indice globale 0..=53 nello spazio canonico
    pub fn global_index(&self) -> usize {
        let layer_offset = (self.layer.0 as usize) * SLOTS_PER_LAYER;
        let local_offset = match self.control {
            ControlId::Button { row, col } => (row as usize) * COL_COUNT + (col as usize),
            ControlId::Encoder { index, event } => {
                BUTTONS_PER_LAYER
                    + (index as usize) * EVENTS_PER_ENCODER
                    + match event {
                        EncoderEvent::Ccw => 0,
                        EncoderEvent::Press => 1,
                        EncoderEvent::Cw => 2,
                    }
            }
        };
        layer_offset + local_offset
    }

    /// Restituisce la lista di tutti i 54 SlotId ordinati canonicamente
    pub fn all_slots() -> Vec<SlotId> {
        let mut slots = Vec::with_capacity(TOTAL_SLOTS);
        for l in 0..LAYER_COUNT as u8 {
            for r in 0..ROW_COUNT as u8 {
                for c in 0..COL_COUNT as u8 {
                    slots.push(SlotId {
                        layer: LayerIndex(l),
                        control: ControlId::Button { row: r, col: c },
                    });
                }
            }
            for enc in 0..ENCODERS_PER_LAYER as u8 {
                for ev in [EncoderEvent::Ccw, EncoderEvent::Press, EncoderEvent::Cw] {
                    slots.push(SlotId {
                        layer: LayerIndex(l),
                        control: ControlId::Encoder {
                            index: enc,
                            event: ev,
                        },
                    });
                }
            }
        }
        slots
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub vendor_id: u16,  // 0x1189
    pub product_id: u16, // 0x8890, 0x8840, 0x8842
    pub encoder_order_swapped: bool,
}

impl Default for DeviceDescriptor {
    fn default() -> Self {
        Self {
            vendor_id: 0x1189,
            product_id: 0x8890,
            encoder_order_swapped: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_exactly_54_unique_slots() {
        let slots = SlotId::all_slots();
        assert_eq!(slots.len(), 54, "Must have exactly 54 total slots");

        let mut unique_slots = HashSet::new();
        let mut unique_indices = HashSet::new();

        for (idx, slot) in slots.iter().enumerate() {
            assert_eq!(
                slot.global_index(),
                idx,
                "Slot global index must match position"
            );
            assert!(unique_slots.insert(*slot), "SlotId must be unique");
            assert!(
                unique_indices.insert(slot.global_index()),
                "Global index must be unique"
            );
        }

        assert_eq!(unique_slots.len(), 54);
        assert_eq!(unique_indices.len(), 54);
    }

    #[test]
    fn test_slots_distribution_per_layer() {
        for layer in 0..3 {
            let mut buttons = 0;
            let mut encoder_events = 0;

            for slot in SlotId::all_slots() {
                if slot.layer.0 == layer {
                    match slot.control {
                        ControlId::Button { .. } => buttons += 1,
                        ControlId::Encoder { .. } => encoder_events += 1,
                    }
                }
            }

            assert_eq!(buttons, 12, "Each layer must have 12 buttons");
            assert_eq!(
                encoder_events, 6,
                "Each layer must have 2 encoders * 3 events = 6 events"
            );
        }
    }
}
