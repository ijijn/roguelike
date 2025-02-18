use super::{Skill, Skills};

#[must_use]
pub const fn attr_bonus(value: i32) -> i32 {
    (value - 10) / 2 // See: https://roll20.net/compendium/dnd5e/Ability%20Scores#content
}

#[must_use]
pub const fn player_hp_per_level(fitness: i32) -> i32 {
    15 + attr_bonus(fitness)
}

#[must_use]
pub const fn player_hp_at_level(fitness: i32, level: i32) -> i32 {
    15 + (player_hp_per_level(fitness) * level)
}

#[must_use]
pub fn npc_hp(fitness: i32, level: i32) -> i32 {
    let mut total = 1;
    for _i in 0..level {
        total += i32::max(1, 8 + attr_bonus(fitness));
    }
    total
}

#[must_use]
pub fn mana_per_level(intelligence: i32) -> i32 {
    i32::max(1, 4 + attr_bonus(intelligence))
}

#[must_use]
pub fn mana_at_level(intelligence: i32, level: i32) -> i32 {
    mana_per_level(intelligence) * level
}

#[must_use]
pub fn skill_bonus(skill: Skill, skills: &Skills) -> i32 {
    if skills.skills.contains_key(&skill) {
        skills.skills[&skill]
    } else {
        -4
    }
}
