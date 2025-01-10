use crate::comp::inventory::PlayerInventory;
use crate::comp::GameEntity;
use crate::world::WorldData;
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_game_base::{AttackSkill, AttackSkillError};

pub struct Attack;

impl Attack {
    pub(crate) fn find_attack_for_player(
        inventory: &PlayerInventory,
    ) -> Result<&'static RefSkillData, AttackSkillError> {
        let weapon = inventory.get_equipment_item(EquipmentSlot::Weapon);
        AttackSkill::get_attack_skill(WorldData::skills(), weapon)
    }

    pub(crate) fn find_attack_for_monster(monster: GameEntity) -> Option<&'static RefSkillData> {
        WorldData::characters()
            .find_id(monster.ref_id)
            .and_then(|chardata| chardata.skills.first())
            .and_then(|skill| WorldData::skills().find_id(*skill))
    }
}
