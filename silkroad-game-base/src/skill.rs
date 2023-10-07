use crate::Item;
use silkroad_data::itemdata::RefItemData;
use silkroad_data::skilldata::RefSkillData;
use silkroad_data::DataMap;
use silkroad_definitions::type_id::{ObjectEquippable, ObjectItem, ObjectType, ObjectWeaponType};
use thiserror::Error;

pub struct AttackSkill;

const PUNCH_SKILL_ID: u32 = 1;

impl AttackSkill {
    pub fn get_range_for_attack(skill: &RefSkillData, weapon: Option<&RefItemData>) -> f32 {
        let skill_range: f32 = skill.range.into();
        let item_range: f32 = weapon.map_or(0.0, |item| {
            item.range.map(|non_zero| non_zero.get().into()).unwrap_or(0.0)
        });
        skill_range + item_range
    }

    pub fn get_attack_skill<'a>(
        skills: &'a DataMap<RefSkillData>,
        weapon: Option<&Item>,
    ) -> Result<&'a RefSkillData, AttackSkillError> {
        let skill = if let Some(weapon) = weapon {
            let item_type = match ObjectType::from_type_id(&weapon.reference.common.type_id) {
                Some(ObjectType::Item(item)) => item,
                _ => return Err(AttackSkillError::NotAWeapon),
            };
            match item_type {
                ObjectItem::Equippable(ObjectEquippable::Weapon(weapon_type)) => match weapon_type {
                    ObjectWeaponType::Sword | ObjectWeaponType::Blade => {
                        skills.find_id(2).ok_or(AttackSkillError::SkillNotFound)?
                    },
                    ObjectWeaponType::Spear | ObjectWeaponType::Glavie => {
                        skills.find_id(40).ok_or(AttackSkillError::SkillNotFound)?
                    },
                    ObjectWeaponType::Bow => skills.find_id(70).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::OneHandSword => skills.find_id(7127).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::TwoHandSword => skills.find_id(7128).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Axe => skills.find_id(7129).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::WarlockStaff => skills.find_id(9069).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Staff => skills.find_id(8454).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Crossbow => skills.find_id(7909).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Dagger => skills.find_id(7910).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::Harp => skills.find_id(9606).ok_or(AttackSkillError::SkillNotFound)?,
                    ObjectWeaponType::ClericRod => skills.find_id(9970).ok_or(AttackSkillError::SkillNotFound)?,
                    _ => return Err(AttackSkillError::UnknownWeapon),
                },
                _ => return Err(AttackSkillError::NotAWeapon),
            }
        } else {
            skills.find_id(PUNCH_SKILL_ID).ok_or(AttackSkillError::SkillNotFound)?
        };
        Ok(skill)
    }
}

#[derive(Error, Debug)]
pub enum AttackSkillError {
    #[error("The item being held is not a weapon")]
    NotAWeapon,
    #[error("The skill requested could not be found")]
    SkillNotFound,
    #[error("The type of weapon was not known")]
    UnknownWeapon,
}
