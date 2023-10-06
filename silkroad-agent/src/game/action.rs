use crate::agent::states;
use crate::agent::states::{Action, ActionDescription, MoveToAction, MoveToPickup, StateTransitionQueue};
use crate::comp::drop::Drop;
use crate::comp::inventory::PlayerInventory;
use crate::comp::net::Client;
use crate::comp::pos::Position;
use crate::comp::GameEntity;
use crate::ext::Navmesh;
use crate::game::world::AttackSkillError;
use crate::input::PlayerInput;
use crate::world::{EntityLookup, WorldData};
use bevy_ecs::prelude::*;
use cgmath::num_traits::Pow;
use cgmath::InnerSpace;
use silkroad_data::skilldata::RefSkillData;
use silkroad_data::DataMap;
use silkroad_definitions::inventory::EquipmentSlot;
use silkroad_definitions::type_id::{ObjectEquippable, ObjectItem, ObjectType, ObjectWeaponType};
use silkroad_game_base::{get_range_for_attack, GlobalLocation, Item, Vector3Ext};
use silkroad_protocol::combat::{
    ActionTarget, DoActionResponseCode, DoActionType, PerformAction, PerformActionError, PerformActionResponse,
};
use std::ops::{Mul, Sub};

const PUNCH_SKILL_ID: u32 = 1;

pub(crate) fn handle_action(
    mut query: Query<(
        &Client,
        &PlayerInput,
        &Position,
        &PlayerInventory,
        &mut StateTransitionQueue,
    )>,
    lookup: Res<EntityLookup>,
    navmesh: Res<Navmesh>,
    target_query: Query<(&GameEntity, &Position)>,
    pickup_query: Query<&Position, With<Drop>>,
) {
    for (client, input, my_pos, inventory, mut state) in query.iter_mut() {
        let Some(ref action) = input.action else {
            continue;
        };

        match action {
            PerformAction::Do(action) => match action {
                DoActionType::Attack { target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Ok((found_target, target_pos)) = target_query.get(target) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let weapon = inventory.get_equipment_item(EquipmentSlot::Weapon);
                        let skill = match get_attack_skill(WorldData::skills(), weapon) {
                            Ok(skill) => skill,
                            Err(err) => match err {
                                AttackSkillError::NotAWeapon | AttackSkillError::UnknownWeapon => {
                                    client.send(PerformActionResponse::Stop(PerformActionError::InvalidWeapon));
                                    continue;
                                },
                                AttackSkillError::SkillNotFound => {
                                    client.send(PerformActionResponse::Stop(PerformActionError::NotLearned));
                                    continue;
                                },
                            },
                        };

                        let description = ActionDescription(skill, states::ActionTarget::Entity(target));
                        let range = get_range_for_attack(skill, weapon.map(|item| item.reference));
                        let range_squared = range.pow(2);
                        let vec_to_target = target_pos.location.to_flat_vec2().sub(my_pos.location.to_flat_vec2());
                        if vec_to_target.magnitude2() <= range_squared {
                            state.request_transition(Action::from(description));
                        } else {
                            let new_target_position = GlobalLocation(vec_to_target.sub(vec_to_target.mul(range)));
                            let new_height = navmesh.height_for(new_target_position).unwrap_or(target_pos.location.y);
                            state.request_transition(MoveToAction(
                                target,
                                new_target_position.with_y(new_height),
                                description,
                            ));
                        }

                        client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                    },
                    _ => continue,
                },
                DoActionType::PickupItem { target } => match target {
                    ActionTarget::Entity(unique_id) => {
                        let Some(target) = lookup.get_entity_for_id(*unique_id) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        let Ok(pos) = pickup_query.get(target) else {
                            client.send(PerformActionResponse::Stop(PerformActionError::InvalidTarget));
                            continue;
                        };

                        client.send(PerformActionResponse::Do(DoActionResponseCode::Success));
                        state.request_transition(MoveToPickup(target, pos.location));
                    },
                    _ => continue,
                },
                DoActionType::UseSkill { .. } => {},
                DoActionType::CancelBuff { .. } => {},
            },
            PerformAction::Stop => {
                // TODO
            },
        }
    }
}

fn get_attack_skill<'a>(
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
