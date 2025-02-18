use super::{EffectSpawner, Entity, World, WorldExt};
use crate::components::{HungerClock, HungerState};

pub fn well_fed(ecs: &World, _damage: &EffectSpawner, target: Entity) {
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        hc.state = HungerState::WellFed;
        hc.duration = 20;
    }
}
