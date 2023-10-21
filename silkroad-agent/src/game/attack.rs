use crate::agent::states;
use crate::agent::states::{Action, ActionDescription, MovementGoal, Moving, StateTransitionQueue};
use crate::comp::inventory::PlayerInventory;
use crate::comp::pos::Position;
use crate::comp::{EntityReference, GameEntity};
use crate::ext::Navmesh;
use crate::world::WorldData;
use cgmath::num_traits::Pow;
use derive_more::Constructor;
use silkroad_data::skilldata::RefSkillData;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_game_base::{AttackSkill, AttackSkillError, Item};

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

#[derive(Constructor)]
pub(crate) struct AttackProcess<'a> {
    state: &'a mut StateTransitionQueue,
    position: &'a Position,
    skill: &'static RefSkillData,
    weapon: Option<&'a Item>,
    target: &'a EntityReference,
    target_pos: &'a Position,
    navmesh: &'a Navmesh,
}

impl AttackProcess<'_> {
    pub(crate) fn try_attack(&mut self) -> Result<(), AttackSkillError> {
        let description = ActionDescription(self.skill, states::ActionTarget::Entity(self.target.0));
        let range = AttackSkill::get_range_for_attack(self.skill, self.weapon.map(|item| item.reference));
        let range_squared = range.pow(2);
        if self.position.distance_to(self.target_pos) <= range_squared {
            self.state.request_transition(Action::from(description));
        } else {
            let new_target_position = self
                .position
                .position()
                .to_location()
                .point_in_line_with_range(self.target_pos.location(), range);
            let new_height = self
                .navmesh
                .height_for(new_target_position)
                .unwrap_or(self.target_pos.position().y);
            self.state
                .request_transition(Moving(MovementGoal::Location(new_target_position.with_y(new_height))));
        }

        Ok(())
    }
}
