// fn get_default_attack_range(
//     player_entity: &GameEntity,
//     inventory: &PlayerInventory,
// ) -> Result<(&'static RefSkillData, f32), AttackSkillError> {
//     let weapon = inventory.weapon();
//     let attack_skill = get_attack_skill(WorldData::skills(), weapon)?;
//     let character_data = WorldData::characters().find_id(player_entity.ref_id).unwrap();
//     let weapon_data = weapon.map(|weapon| weapon.reference);
//     let range = vec![
//         attack_skill.range,
//         character_data.base_range,
//         weapon_data
//             .and_then(|weapon| weapon.range)
//             .map(|r| r.get())
//             .unwrap_or(0),
//     ]
//     .into_iter()
//     .max()
//     .unwrap_or(0) as f32;
//     Ok((attack_skill, range))
// }

pub(crate) enum AttackSkillError {
    NotAWeapon,
    SkillNotFound,
    UnknownWeapon,
}
