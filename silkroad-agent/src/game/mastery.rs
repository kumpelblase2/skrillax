use crate::comp::exp::{Leveled, SP};
use crate::comp::mastery::MasteryKnowledge;
use crate::comp::net::Client;
use crate::comp::player::CharacterRace;
use crate::comp::skill::SkillBook;
use crate::config::GameConfig;
use crate::input::PlayerInputEvent;
use crate::world::WorldData;
use bevy::prelude::*;
use silkroad_game_base::Race;
use silkroad_protocol::skill::{
    LearnSkill, LearnSkillResponse, LevelUpMastery, LevelUpMasteryError, LevelUpMasteryResponse,
};

pub(crate) fn handle_mastery_levelup(
    mut query: Query<(&Client, &CharacterRace, &Leveled, &mut MasteryKnowledge, &mut SP)>,
    config: Res<GameConfig>,
    mut reader: MessageReader<PlayerInputEvent<LevelUpMastery>>,
) {
    let masteries = WorldData::masteries();
    let levels = WorldData::levels();
    for event in reader.read() {
        let Ok((client, race, level, mut knowledge, mut sp)) = query.get_mut(event.player) else {
            continue;
        };

        let mastery_levelup = &event.input;

        if masteries.find_id(mastery_levelup.mastery).is_none() {
            client.send(LevelUpMasteryResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
            continue;
        }

        let current_level = knowledge.level_of(mastery_levelup.mastery).unwrap_or(0);

        let per_level_cap = match race.inner() {
            Race::European => config.masteries.european_per_level,
            Race::Chinese => config.masteries.chinese_per_level,
        };

        let current_cap = u16::from(level.current_level()) * per_level_cap;
        let total_mastery_levels = knowledge.total();
        if total_mastery_levels >= current_cap {
            client.send(LevelUpMasteryResponse::Failure(LevelUpMasteryError::ReachedTotalLimit));
            continue;
        }

        let required_sp = levels.get_mastery_sp_for_level(current_level).unwrap_or(0);

        if sp.current() < required_sp {
            client.send(LevelUpMasteryResponse::Failure(LevelUpMasteryError::InsufficientSP));
            continue;
        }

        if required_sp > 0 {
            sp.spend(required_sp);
        }

        knowledge.level_mastery_by(mastery_levelup.mastery, mastery_levelup.amount);
    }
}

pub(crate) fn learn_skill(
    mut query: Query<(&Client, &MasteryKnowledge, &mut SkillBook, &CharacterRace, &mut SP)>,
    mut reader: MessageReader<PlayerInputEvent<LearnSkill>>,
) {
    for event in reader.read() {
        let Ok((client, mastery_knowledge, mut skill_book, race, mut sp)) = query.get_mut(event.player) else {
            continue;
        };

        let learn = &event.input;
        let Some(skill) = WorldData::skills().find_id(learn.0) else {
            client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
            continue;
        };

        if skill.race != 3 && skill.race != race.inner().as_skill_origin() {
            client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
            continue;
        }

        if skill.sp > sp.current() {
            client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
            continue;
        }

        if let Some(mastery_id) = &skill.mastery {
            let mastery_id = mastery_id.get();
            let Some(mastery_level) = mastery_knowledge.level_of(mastery_id as u32) else {
                client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
                continue;
            };

            if let Some(required_level) = &skill.mastery_level {
                if required_level.get() > mastery_level {
                    client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
                    continue;
                }
            }
        }

        if !skill_book.has_required_skills_for(skill) {
            client.send(LearnSkillResponse::Failure(LevelUpMasteryError::InsufficientSP)); // TODO
            continue;
        }

        sp.spend(skill.sp);
        skill_book.learn_skill(skill);
        client.send(LearnSkillResponse::Success(learn.0));
    }
}
