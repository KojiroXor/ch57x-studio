use serde::{Deserialize, Serialize};

pub const TOTAL_TRIGGER_POOL_SIZE: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TriggerBank {
    BankA, // ctrl-alt-shift-f1..f11, 0 (0..=11)
    BankB, // ctrl-alt-shift-1..9, a, b, c (12..=23)
    BankC, // ctrl-alt-shift-d..o (24..=35)
    BankD, // ctrl-alt-shift-p..z, minus (36..=47)
    BankE, // ctrl-alt-win-f1..f11, 0 (48..=59)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PhantomTrigger {
    pub bank: TriggerBank,
    pub index: u8,      // 0..=11
    pub key_number: u8, // 13..=24 per retro-compatibilità test
}

impl PhantomTrigger {
    pub fn new(bank: TriggerBank, index_or_key: u8) -> Option<Self> {
        let index = if index_or_key < 12 {
            index_or_key
        } else if (13..=24).contains(&index_or_key) {
            index_or_key - 13
        } else {
            return None;
        };
        Some(Self {
            bank,
            index,
            key_number: 13 + index,
        })
    }

    /// Ritorna il token del tasto base (es. "f1", "0", "1", "a", "minus")
    pub fn key_token(&self) -> &'static str {
        match self.bank {
            TriggerBank::BankA => match self.index {
                0 => "f1",
                1 => "f2",
                2 => "f3",
                3 => "f4",
                4 => "f5",
                5 => "f6",
                6 => "f7",
                7 => "f8",
                8 => "f9",
                9 => "f10",
                10 => "f11",
                11 => "0",
                _ => "f1",
            },
            TriggerBank::BankB => match self.index {
                0 => "1",
                1 => "2",
                2 => "3",
                3 => "4",
                4 => "5",
                5 => "6",
                6 => "7",
                7 => "8",
                8 => "9",
                9 => "a",
                10 => "b",
                11 => "c",
                _ => "1",
            },
            TriggerBank::BankC => match self.index {
                0 => "d",
                1 => "e",
                2 => "f",
                3 => "g",
                4 => "h",
                5 => "i",
                6 => "j",
                7 => "k",
                8 => "l",
                9 => "m",
                10 => "n",
                11 => "o",
                _ => "d",
            },
            TriggerBank::BankD => match self.index {
                0 => "p",
                1 => "q",
                2 => "r",
                3 => "s",
                4 => "t",
                5 => "u",
                6 => "v",
                7 => "w",
                8 => "x",
                9 => "y",
                10 => "z",
                11 => "minus",
                _ => "p",
            },
            TriggerBank::BankE => match self.index {
                0 => "f1",
                1 => "f2",
                2 => "f3",
                3 => "f4",
                4 => "f5",
                5 => "f6",
                6 => "f7",
                7 => "f8",
                8 => "f9",
                9 => "f10",
                10 => "f11",
                11 => "0",
                _ => "f1",
            },
        }
    }

    /// Ritorna la sintassi esatta attesa da ch57x-keyboard-tool (es. "ctrl-alt-shift-f1", "ctrl-alt-win-0")
    pub fn yaml_token(&self) -> String {
        let key = self.key_token();
        match self.bank {
            TriggerBank::BankA | TriggerBank::BankB | TriggerBank::BankC | TriggerBank::BankD => {
                format!("ctrl-alt-shift-{}", key)
            }
            TriggerBank::BankE => {
                format!("ctrl-alt-win-{}", key)
            }
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
        base + (self.index as usize)
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
            index: offset as u8,
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
        assert_eq!(
            triggers.len(),
            60,
            "Must have exactly 60 candidate triggers"
        );

        let mut unique_yaml = HashSet::new();
        let mut unique_indices = HashSet::new();

        for (idx, trigger) in triggers.iter().enumerate() {
            assert_eq!(trigger.pool_index(), idx);
            assert!(
                unique_yaml.insert(trigger.yaml_token()),
                "YAML tokens must be unique: {}",
                trigger.yaml_token()
            );
            assert!(
                unique_indices.insert(trigger.pool_index()),
                "Pool indices must be unique"
            );
        }

        assert_eq!(unique_yaml.len(), 60);
        assert_eq!(unique_indices.len(), 60);
    }

    #[test]
    fn test_trigger_banks() {
        // Bank A: ctrl-alt-shift-f1..f11, 0
        assert_eq!(
            PhantomTrigger::from_pool_index(0).unwrap().yaml_token(),
            "ctrl-alt-shift-f1"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(10).unwrap().yaml_token(),
            "ctrl-alt-shift-f11"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(11).unwrap().yaml_token(),
            "ctrl-alt-shift-0"
        );

        // Bank B: ctrl-alt-shift-1..9, a, b, c
        assert_eq!(
            PhantomTrigger::from_pool_index(12).unwrap().yaml_token(),
            "ctrl-alt-shift-1"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(20).unwrap().yaml_token(),
            "ctrl-alt-shift-9"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(21).unwrap().yaml_token(),
            "ctrl-alt-shift-a"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(23).unwrap().yaml_token(),
            "ctrl-alt-shift-c"
        );

        // Bank C: ctrl-alt-shift-d..o
        assert_eq!(
            PhantomTrigger::from_pool_index(24).unwrap().yaml_token(),
            "ctrl-alt-shift-d"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(35).unwrap().yaml_token(),
            "ctrl-alt-shift-o"
        );

        // Bank D: ctrl-alt-shift-p..z, minus
        assert_eq!(
            PhantomTrigger::from_pool_index(36).unwrap().yaml_token(),
            "ctrl-alt-shift-p"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(46).unwrap().yaml_token(),
            "ctrl-alt-shift-z"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(47).unwrap().yaml_token(),
            "ctrl-alt-shift-minus"
        );

        // Bank E: ctrl-alt-win-f1..f11, 0
        assert_eq!(
            PhantomTrigger::from_pool_index(48).unwrap().yaml_token(),
            "ctrl-alt-win-f1"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(58).unwrap().yaml_token(),
            "ctrl-alt-win-f11"
        );
        assert_eq!(
            PhantomTrigger::from_pool_index(59).unwrap().yaml_token(),
            "ctrl-alt-win-0"
        );
    }
}
