use crate::{
    effects::{add_effect, aoe_tiles, EffectType, Targets},
    AreaOfEffect, EntityMoved, EntryTrigger, Map, Name, Position,
};
use specs::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        ReadStorage<'a, AreaOfEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, names, entities, area_of_effect) =
            data;

        // Iterate the entities that moved and their final position
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            crate::spatial::for_each_tile_content(idx, |entity_id| {
                if entity != entity_id {
                    // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            // We triggered it
                            let name = names.get(entity_id);
                            if let Some(name) = name {
                                crate::gamelog::Logger::new()
                                    .item_name(&name.name)
                                    .append(&"triggers!")
                                    .log();
                            }

                            // Call the effects system
                            add_effect(
                                Some(entity),
                                EffectType::TriggerFire { trigger: entity_id },
                                area_of_effect.get(entity_id).map_or(
                                    Targets::Tile {
                                        tile_idx: idx as i32,
                                    },
                                    |aoe| Targets::Tiles {
                                        tiles: aoe_tiles(
                                            &map,
                                            rltk::Point::new(pos.x, pos.y),
                                            aoe.radius,
                                        ),
                                    },
                                ),
                            );
                        }
                    }
                }
            });
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
