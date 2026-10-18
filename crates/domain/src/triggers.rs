use serde::{Deserialize, Serialize};

pub const TOTAL_TRIGGER_POOL_SIZE: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TriggerBank {
    BankA, // None + F13..F24 (0..=11)
    BankB, // Ctrl + F13..F24 (12..=23)
    BankC, // Shift + F13..F24 (24..=35)
    BankD, // Alt + F13..F24 (36..=47)
    BankE, // Ctrl+Shift + F13..F24 (48..=59)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PhantomTrigger {
    pub bank: TriggerBank,
    pub key_number: u8, // 13..=24
}

impl PhantomTrigger {
    pub fn new(bank: TriggerBank, key_number: u8) -> Option<Self> {
        if (13..=24).contains(&key_number) {
            Some(Self { bank, key_number })
        } else {
            None
        }
    }

    /// Ritorna la sintassi esatta attesa da ch57x-keyboard-tool (es. "f13", "ctrl-f13", "ctrl-shift-f24")
    pub fn yaml_token(&self) -> String {
        let key_str = format!("f{}", self.key_number);
        match self.bank {
            TriggerBank::BankA => key_str,
            TriggerBank::BankB => format!("ctrl-{}", key_str),
            TriggerBank::BankC => format!("shift-{}", key_str),
            TriggerBank::BankD => format!("alt-{}", key_str),
            TriggerBank::BankE => format!("ctrl-shift-{}", key_str),
        }
    }

    /// Ritorna l'indice nel pool (0..=59)
    pub fn pool_index(&self) -> usize {
        let base = match self.bank {
            TriggerBank::BankA => 0,
            TriggerBank::BankB => 12,
            TriggerBank::BankC => 24,
            TriggerBank::BankD => 36,
            TriggerBank::BankE => 48,
        };
        base + (self.key_number - 13) as usize
    }

    /// Ritorna il trigger dato l'indice nel pool 0..=59
    pub fn from_pool_index(idx: usize) -> Option<Self> {
        if idx >= TOTAL_TRIGGER_POOL_SIZE {
            return None;
        }
        let (bank, offset) = match idx {
            0..=11 => (TriggerBank::BankA, idx),
            12..=23 => (TriggerBank::BankB, idx - 12),
            24..=35 => (TriggerBank::BankC, idx - 24),
            36..=47 => (TriggerBank::BankD, idx - 36),
            48..=59 => (TriggerBank::BankE, idx - 48),
            _ => unreachable!(),
        };
        Some(Self {
            bank,
            key_number: (13 + offset) as u8,
        })
    }

    /// Ritorna la lista completa di tutti i 60 trigger nell'ordine canonico del pool
    pub fn all_candidates() -> Vec<PhantomTrigger> {
        (0..TOTAL_TRIGGER_POOL_SIZE)
            .filter_map(Self::from_pool_index)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_exactly_60_candidate_triggers() {
        let triggers = PhantomTrigger::all_candidates();
        assert_eq!(triggers.len(), 60, "Must have exactly 60 candidate triggers");

        let mut unique_yaml = HashSet::new();
        let mut unique_indices = HashSet::new();

        for (idx, trigger) in triggers.iter().enumerate() {
            assert_eq!(trigger.pool_index(), idx);
            assert!(unique_yaml.insert(trigger.yaml_token()), "YAML tokens must be unique");
            assert!(unique_indices.insert(trigger.pool_index()), "Pool indices must be unique");
        }

        assert_eq!(unique_yaml.len(), 60);
        assert_eq!(unique_indices.len(), 60);
    }

    #[test]
    fn test_trigger_banks() {
        // Bank A: f13..f24
        assert_eq!(PhantomTrigger::from_pool_index(0).unwrap().yaml_token(), "f13");
        assert_eq!(PhantomTrigger::from_pool_index(11).unwrap().yaml_token(), "f24");

        // Bank B: ctrl-f13..ctrl-f24
        assert_eq!(PhantomTrigger::from_pool_index(12).unwrap().yaml_token(), "ctrl-f13");
        assert_eq!(PhantomTrigger::from_pool_index(23).unwrap().yaml_token(), "ctrl-f24");

        // Bank C: shift-f13..shift-f24
        assert_eq!(PhantomTrigger::from_pool_index(24).unwrap().yaml_token(), "shift-f13");
        assert_eq!(PhantomTrigger::from_pool_index(35).unwrap().yaml_token(), "shift-f24");

        // Bank D: alt-f13..alt-f24
        assert_eq!(PhantomTrigger::from_pool_index(36).unwrap().yaml_token(), "alt-f13");
        assert_eq!(PhantomTrigger::from_pool_index(47).unwrap().yaml_token(), "alt-f24");

        // Bank E: ctrl-shift-f13..ctrl-shift-f24
        assert_eq!(PhantomTrigger::from_pool_index(48).unwrap().yaml_token(), "ctrl-shift-f13");
        assert_eq!(PhantomTrigger::from_pool_index(59).unwrap().yaml_token(), "ctrl-shift-f24");
    }
}
