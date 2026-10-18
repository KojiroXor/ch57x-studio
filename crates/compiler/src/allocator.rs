//! Allocatore deterministico dei 60 trigger sui 54 slot
//!
//! Dato un Profile, assegna deterministicamente i trigger dai 60 candidati
//! (banchi A..E) a tutti gli slot che hanno `SlotBinding::HostActionRef`.
//! Preserva i trigger gia' assegnati e alloca il primo libero per i nuovi slot.
//! Verifica l'assenza di collisioni.

use ch57x_domain::actions::SlotBinding;
use ch57x_domain::errors::DomainError;
use ch57x_domain::hardware::TOTAL_SLOTS;
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::{PhantomTrigger, TOTAL_TRIGGER_POOL_SIZE};
use std::collections::{HashMap, HashSet};

/// Assegna deterministicamente i trigger dal pool di 60 candidati a tutti gli slot
/// con binding `SlotBinding::HostActionRef`.
///
/// Regole:
/// 1. I trigger gia' assegnati a slot ancora validi vengono preservati.
/// 2. I trigger assegnati a slot non piu' `HostActionRef` vengono rilasciati.
/// 3. Per i nuovi slot `HostActionRef`, viene allocato il primo trigger libero nel pool (0..=59).
/// 4. Verifica che non ci siano collisioni (due slot con lo stesso trigger).
pub fn allocate_triggers(profile: &mut Profile) -> Result<(), DomainError> {
    // 1. Pulisci le allocazioni per slot che non sono piu' HostActionRef o fuori range
    let host_slots: HashSet<usize> = profile
        .slots
        .iter()
        .filter_map(|(&idx, cfg)| {
            if idx < TOTAL_SLOTS && matches!(cfg.binding, SlotBinding::HostActionRef(_)) {
                Some(idx)
            } else {
                None
            }
        })
        .collect();

    profile
        .trigger_allocation
        .retain(|slot_idx, _| host_slots.contains(slot_idx));

    // 2. Mappa dei trigger attualmente in uso: pool_index -> slot_idx
    let mut used_pool_indices: HashMap<usize, usize> = HashMap::new();
    for (&slot_idx, trigger) in &profile.trigger_allocation {
        let p_idx = trigger.pool_index();
        if p_idx >= TOTAL_TRIGGER_POOL_SIZE {
            return Err(DomainError::InvalidTrigger(format!(
                "Trigger index {} out of range for slot {}",
                p_idx, slot_idx
            )));
        }
        if let Some(existing_slot) = used_pool_indices.insert(p_idx, slot_idx) {
            return Err(DomainError::Validation(format!(
                "Trigger collision: slots {} and {} both claim pool index {} ({})",
                existing_slot,
                slot_idx,
                p_idx,
                trigger.yaml_token()
            )));
        }
    }

    // 3. Alloca i trigger per gli slot HostActionRef che non ne hanno ancora uno
    // Ordine canonico: 0..TOTAL_SLOTS per garantire determinismo assoluto
    let mut next_pool_idx = 0;
    for slot_idx in 0..TOTAL_SLOTS {
        if host_slots.contains(&slot_idx) && !profile.trigger_allocation.contains_key(&slot_idx) {
            // Trova il primo pool_index non utilizzato
            while next_pool_idx < TOTAL_TRIGGER_POOL_SIZE
                && used_pool_indices.contains_key(&next_pool_idx)
            {
                next_pool_idx += 1;
            }

            if next_pool_idx >= TOTAL_TRIGGER_POOL_SIZE {
                return Err(DomainError::Validation(format!(
                    "Trigger pool exhausted (maximum {} triggers), cannot allocate for slot {}",
                    TOTAL_TRIGGER_POOL_SIZE, slot_idx
                )));
            }

            let trigger = PhantomTrigger::from_pool_index(next_pool_idx).ok_or_else(|| {
                DomainError::InvalidTrigger(format!(
                    "Failed to create PhantomTrigger for pool index {}",
                    next_pool_idx
                ))
            })?;

            profile.trigger_allocation.insert(slot_idx, trigger);
            used_pool_indices.insert(next_pool_idx, slot_idx);
            next_pool_idx += 1;
        }
    }

    // 4. Verifica finale di consistenza e assenza di collisioni
    verify_trigger_allocations(profile)?;

    Ok(())
}

/// Verifica l'assenza di collisioni e la validita' dell'allocazione trigger
pub fn verify_trigger_allocations(profile: &Profile) -> Result<(), DomainError> {
    let mut seen_triggers: HashMap<String, usize> = HashMap::new();
    let mut seen_indices: HashMap<usize, usize> = HashMap::new();

    for (&slot_idx, trigger) in &profile.trigger_allocation {
        if slot_idx >= TOTAL_SLOTS {
            return Err(DomainError::InvalidSlot(slot_idx));
        }

        let p_idx = trigger.pool_index();
        if p_idx >= TOTAL_TRIGGER_POOL_SIZE {
            return Err(DomainError::InvalidTrigger(format!(
                "Trigger {} has invalid pool index {}",
                trigger.yaml_token(),
                p_idx
            )));
        }

        let token = trigger.yaml_token();
        if let Some(other_slot) = seen_triggers.insert(token.clone(), slot_idx) {
            return Err(DomainError::Validation(format!(
                "Trigger collision: slots {} and {} both use trigger '{}'",
                other_slot, slot_idx, token
            )));
        }

        if let Some(other_slot) = seen_indices.insert(p_idx, slot_idx) {
            return Err(DomainError::Validation(format!(
                "Trigger index collision: slots {} and {} both use pool index {}",
                other_slot, slot_idx, p_idx
            )));
        }

        // Verifica che lo slot sia effettivamente un HostActionRef
        if let Some(slot_cfg) = profile.slots.get(&slot_idx) {
            if !matches!(slot_cfg.binding, SlotBinding::HostActionRef(_)) {
                return Err(DomainError::Validation(format!(
                    "Slot {} has trigger allocation '{}' but its binding is not HostActionRef",
                    slot_idx, token
                )));
            }
        }
    }

    // Verifica che tutti gli slot HostActionRef abbiano un trigger
    for (&slot_idx, slot_cfg) in &profile.slots {
        if matches!(slot_cfg.binding, SlotBinding::HostActionRef(_))
            && !profile.trigger_allocation.contains_key(&slot_idx)
        {
            return Err(DomainError::Validation(format!(
                "Slot {} has HostActionRef binding but no trigger allocated",
                slot_idx
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ch57x_domain::actions::ActionId;
    use ch57x_domain::hardware::SlotId;
    use ch57x_domain::triggers::TriggerBank;

    #[test]
    fn test_allocate_all_54_slots() {
        let mut profile = Profile::default();
        // Configura tutti i 54 slot come HostActionRef
        for slot in SlotId::all_slots() {
            let idx = slot.global_index();
            if let Some(cfg) = profile.slots.get_mut(&idx) {
                cfg.binding = SlotBinding::HostActionRef(ActionId::new_v4());
            }
        }

        allocate_triggers(&mut profile).expect("Allocation of all 54 slots must succeed");

        assert_eq!(profile.trigger_allocation.len(), 54);
        let mut tokens = HashSet::new();
        for trigger in profile.trigger_allocation.values() {
            assert!(
                tokens.insert(trigger.yaml_token()),
                "All 54 triggers must be unique"
            );
        }
        assert_eq!(tokens.len(), 54);
    }

    #[test]
    fn test_preserve_existing_allocations() {
        let mut profile = Profile::default();
        let slot_0 = 0;
        let slot_1 = 1;

        // Imposta slot 0 con un trigger specifico di Banco D (alt-f13)
        let custom_trigger = PhantomTrigger::new(TriggerBank::BankD, 13).unwrap();
        profile.trigger_allocation.insert(slot_0, custom_trigger);

        profile.slots.get_mut(&slot_0).unwrap().binding =
            SlotBinding::HostActionRef(ActionId::new_v4());
        profile.slots.get_mut(&slot_1).unwrap().binding =
            SlotBinding::HostActionRef(ActionId::new_v4());

        allocate_triggers(&mut profile).expect("Allocation should succeed");

        // Slot 0 deve aver preservato il suo trigger (alt-f13, pool_index 36)
        assert_eq!(
            profile.trigger_allocation.get(&slot_0).unwrap().yaml_token(),
            "alt-f13"
        );
        // Slot 1 deve aver ricevuto il primo libero (f13, pool_index 0)
        assert_eq!(
            profile.trigger_allocation.get(&slot_1).unwrap().yaml_token(),
            "f13"
        );
    }

    #[test]
    fn test_freed_slots_are_cleaned_up() {
        let mut profile = Profile::default();
        let slot_0 = 0;

        profile.slots.get_mut(&slot_0).unwrap().binding =
            SlotBinding::HostActionRef(ActionId::new_v4());
        allocate_triggers(&mut profile).unwrap();
        assert_eq!(profile.trigger_allocation.len(), 1);

        // Disabilita lo slot
        profile.slots.get_mut(&slot_0).unwrap().binding = SlotBinding::Disabled;
        allocate_triggers(&mut profile).unwrap();
        assert_eq!(profile.trigger_allocation.len(), 0);
    }

    #[test]
    fn test_collision_detection() {
        let mut profile = Profile::default();
        let trig = PhantomTrigger::new(TriggerBank::BankA, 13).unwrap();

        profile.slots.get_mut(&0).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());
        profile.slots.get_mut(&1).unwrap().binding = SlotBinding::HostActionRef(ActionId::new_v4());

        // Assegna volutamente lo stesso trigger a due slot diversi
        profile.trigger_allocation.insert(0, trig);
        profile.trigger_allocation.insert(1, trig);

        let result = allocate_triggers(&mut profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("collision"));
    }
}
