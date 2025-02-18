use super::{
    AreaOfEffect, EquipmentChanged, IdentifiedItem, Map, Name, WantsToCastSpell, WantsToUseItem,
};
use crate::effects::{add_effect, aoe_tiles, EffectType, Targets};
use specs::prelude::*;

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, IdentifiedItem>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            entities,
            mut wants_use,
            names,
            aoe,
            mut dirty,
            mut identified_item,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert");

            // Identify
            if entity == *player_entity {
                identified_item
                    .insert(
                        entity,
                        IdentifiedItem {
                            name: names.get(useitem.item).unwrap().name.clone(),
                        },
                    )
                    .expect("Unable to insert");
            }

            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::ItemUse { item: useitem.item },
                useitem.target.map_or_else(
                    || Targets::Single {
                        target: *player_entity,
                    },
                    |target| {
                        aoe.get(useitem.item).map_or_else(
                            || Targets::Tile {
                                tile_idx: map.xy_idx(target.x, target.y) as i32,
                            },
                            |aoe| Targets::Tiles {
                                tiles: aoe_tiles(&map, target, aoe.radius),
                            },
                        )
                    },
                ),
            );
        }

        wants_use.clear();
    }
}

pub struct SpellUseSystem {}

impl<'a> System<'a> for SpellUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToCastSpell>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, IdentifiedItem>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            map,
            entities,
            mut wants_use,
            names,
            aoe,
            mut dirty,
            mut identified_item,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to insert");

            // Identify
            if entity == *player_entity {
                identified_item
                    .insert(
                        entity,
                        IdentifiedItem {
                            name: names.get(useitem.spell).unwrap().name.clone(),
                        },
                    )
                    .expect("Unable to insert");
            }

            // Call the effects system
            add_effect(
                Some(entity),
                EffectType::SpellUse {
                    spell: useitem.spell,
                },
                useitem.target.map_or_else(
                    || Targets::Single {
                        target: *player_entity,
                    },
                    |target| {
                        aoe.get(useitem.spell).map_or_else(
                            || Targets::Tile {
                                tile_idx: map.xy_idx(target.x, target.y) as i32,
                            },
                            |aoe| Targets::Tiles {
                                tiles: aoe_tiles(&map, target, aoe.radius),
                            },
                        )
                    },
                ),
            );
        }

        wants_use.clear();
    }
}
