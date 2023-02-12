use silkroad_data::itemdata::RefItemData;
use silkroad_data::skilldata::RefSkillData;

pub fn get_range_for_attack(skill: &RefSkillData, weapon: Option<&RefItemData>) -> f32 {
    let skill_range: f32 = skill.range.into();
    let item_range: f32 = weapon.map_or(0.0, |item| {
        item.range.map(|non_zero| non_zero.get().into()).unwrap_or(0.0)
    });
    skill_range + item_range
}
