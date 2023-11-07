use crate::comp::exp::{Leveled, SP};
use crate::comp::mastery::MasteryKnowledge;
use crate::comp::net::Client;
use crate::comp::player::Player;
use crate::config::GameConfig;
use crate::input::PlayerInput;
use crate::world::WorldData;
use bevy_ecs::prelude::*;
use silkroad_game_base::Race;
use silkroad_protocol::skill::{LearnSkillResponse, LevelUpMasteryError, LevelUpMasteryResponse};

pub(crate) fn handle_mastery_levelup(
    mut query: Query<(
        &Client,
        &Player,
        &Leveled,
        &mut MasteryKnowledge,
        &mut SP,
        &mut PlayerInput,
    )>,
    config: Res<GameConfig>,
) {
    let masteries = WorldData::masteries();
    let levels = WorldData::levels();
    for (client, player, level, mut knowledge, mut sp, mut input) in query.iter_mut() {
        if let Some(mastery_levelup) = input.mastery.take() {
            if masteries.find_id(mastery_levelup.mastery).is_none() {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            }

            let current_level = knowledge.level_of(mastery_levelup.mastery).unwrap_or(0);

            let per_level_cap = match player.character.race {
                Race::European => config.masteries.european_per_level,
                Race::Chinese => config.masteries.chinese_per_level,
            };

            let current_cap = u16::from(level.current_level()) * per_level_cap;
            let total_mastery_levels = knowledge.total();
            if total_mastery_levels >= current_cap {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::ReachedTotalLimit));
                continue;
            }

            let required_sp = levels.get_mastery_sp_for_level(current_level).unwrap_or(0);

            if sp.current() < required_sp {
                client.send(LevelUpMasteryResponse::Error(LevelUpMasteryError::InsufficientSP));
                continue;
            }

            if required_sp > 0 {
                sp.spend(required_sp);
            }

            knowledge.level_mastery_by(mastery_levelup.mastery, mastery_levelup.amount);
        }
    }
}

pub(crate) fn learn_skill(mut query: Query<(&Client, &MasteryKnowledge, &mut Player, &mut PlayerInput, &mut SP)>) {
    for (client, mastery_knowledge, mut player, mut input, mut sp) in query.iter_mut() {
        if let Some(learn) = input.skill_add.take() {
            let Some(skill) = WorldData::skills().find_id(learn.0) else {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            };

            if skill.race != 3 && skill.race != player.character.race.as_skill_origin() {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            }

            if skill.sp > sp.current() {
                client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            }

            if let Some(mastery_id) = &skill.mastery {
                let mastery_id = mastery_id.get();
                let Some(mastery_level) = mastery_knowledge.level_of(mastery_id as u32) else {
                    client.send(LearnSkillResponse::Error(LevelUpMasteryError::InsufficientSP)); // TODO
                    continue;
                };

                if let Some(required_level) = &skill.mastery_level {
                    if required_level.get() > mastery_level {
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

            sp.spend(skill.sp);

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
