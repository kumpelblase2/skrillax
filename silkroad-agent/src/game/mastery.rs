use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::config::GameConfig;
use crate::input::PlayerInput;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use silkroad_game_base::Race;
use silkroad_protocol::skill::{LearnSkillResponse, LevelUpMasteryError, LevelUpMasteryResponse};
use silkroad_protocol::world::CharacterPointsUpdate;

pub(crate) fn handle_mastery_levelup(
    mut query: Query<(&Client, &mut Player, &mut PlayerInput)>,
    config: Res<GameConfig>,
) {
    let masteries = WorldData::masteries();
    let levels = WorldData::levels();
    for (client, mut player, mut input) in query.iter_mut() {
        if let Some(mastery_levelup) = input.mastery.take() {
            let Some(mastery) = masteries.find_id(mastery_levelup.mastery) else {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            };

            let current_level = player
                .character
                .masteries
                .iter()
                .find(|(mastery, _)| mastery.ref_id == mastery_levelup.mastery as u16)
                .map(|(_, level)| level)
                .copied()
                .unwrap_or(0);

            let per_level_cap = match player.character.race {
                Race::European => config.masteries.european_per_level,
                Race::Chinese => config.masteries.chinese_per_level,
            };

            let current_cap = usize::from(player.character.level) * per_level_cap;
            let total_mastery_levels = player
                .character
                .masteries
                .iter()
                .map(|(_, level)| usize::from(*level))
                .sum::<usize>();
            if total_mastery_levels >= current_cap {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::ReachedTotalLimit));
                continue;
            }

            let required_sp = levels.get_mastery_sp_for_level(current_level).unwrap_or(0);

            if player.character.sp < required_sp {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP));
                continue;
            }

            if required_sp > 0 {
                player.character.sp -= required_sp;
                client.send(CharacterPointsUpdate::SP {
                    amount: player.character.sp,
                    display: false,
                });
            }

            let next_level = current_level.checked_add(1).unwrap_or(u8::MAX);
            if let Some(position) = player
                .character
                .masteries
                .iter()
                .position(|(mastery, _)| mastery.ref_id == mastery_levelup.mastery as u16)
            {
                player.character.masteries[position] = (mastery, next_level);
            } else {
                player.character.masteries.push((mastery, next_level));
            };
            client.send(LevelUpMasteryResponse::Success {
                mastery: mastery_levelup.mastery,
                new_level: next_level,
            });
        }
    }
}

pub(crate) fn learn_skill(mut query: Query<(&Client, &mut Player, &mut PlayerInput)>) {
    for (client, mut player, mut input) in query.iter_mut() {
        if let Some(learn) = input.skill_add.take() {
            let Some(skill) = WorldData::skills().find_id(learn.0) else {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            };

            if skill.race != 3 && skill.race != player.character.race.as_skill_origin() {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            }

            if skill.sp > player.character.sp {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            }

            if let Some(mastery_id) = &skill.mastery {
                let mastery_id = mastery_id.get();
                let Some((_, mastery_level)) = player
                    .character
                    .masteries
                    .iter()
                    .find(|(mastery, _)| mastery.ref_id == mastery_id)
                else {
                    client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                    continue;
                };

                if let Some(required_level) = &skill.mastery_level {
                    if required_level.get() > *mastery_level {
                        client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                        continue;
                    }
                }
            }

            for required_skill in skill.required_skills.iter().filter(|skill| skill.group != 0) {
                if !player
                    .character
                    .skills
                    .iter()
                    .any(|skill| skill.group == required_skill.group && skill.level >= required_skill.level)
                {
                    client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                    continue;
                }
            }

            player.character.sp -= skill.sp;
            client.send(CharacterPointsUpdate::SP {
                amount: player.character.sp,
                display: false,
            });

            if let Some(pos) = player
                .character
                .skills
                .iter()
                .position(|existing_skill| existing_skill.group == skill.group)
            {
                player.character.skills[pos] = skill;
            } else {
                player.character.skills.push(skill);
            }
            client.send(LearnSkillResponse::Success(learn.0));
        }
    }
}
