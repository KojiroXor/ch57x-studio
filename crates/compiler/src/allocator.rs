//! Allocatore deterministico dei 60 trigger sui 54 slot

use ch57x_domain::hardware::TOTAL_SLOTS;
use ch57x_domain::profile::Profile;
use ch57x_domain::triggers::{PhantomTrigger, TOTAL_TRIGGER_POOL_SIZE};
use std::collections::HashMap;

pub fn allocate_triggers(profile: &mut Profile) {
    let mut used_indices = HashMap::new();
    for (idx, trigger) in &profile.trigger_allocation {
        used_indices.insert(trigger.pool_index(), *idx);
    }

    let mut next_pool_idx = 0;
    for slot_idx in 0..TOTAL_SLOTS {
        if let Some(slot_cfg) = profile.slots.get(&slot_idx) {
            if matches!(slot_cfg.binding, ch57x_domain::actions::SlotBinding::HostActionRef(_)) {
                if !profile.trigger_allocation.contains_key(&slot_idx) {
                    while next_pool_idx < TOTAL_TRIGGER_POOL_SIZE && used_indices.contains_key(&next_pool_idx) {
                        next_pool_idx += 1;
                    }
                    if let Some(trig) = PhantomTrigger::from_pool_index(next_pool_idx) {
                        profile.trigger_allocation.insert(slot_idx, trig);
                        used_indices.insert(next_pool_idx, slot_idx);
                        next_pool_idx += 1;
                    }
                }
            }
        }
    }
}
