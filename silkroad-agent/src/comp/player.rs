use crate::agent::component::{Agent, MovementState};
use crate::agent::goal::GoalTracker;
use crate::agent::state::AgentStateQueue;
use crate::comp::damage::DamageReceiver;
use crate::comp::exp::{Experienced, Leveled, SP};
use crate::comp::gold::GoldPouch;
use crate::comp::inventory::PlayerInventory;
use crate::comp::mastery::MasteryKnowledge;
use crate::comp::pos::Position;
use crate::comp::skill::{Hotbar, SkillBook};
use crate::comp::visibility::Visibility;
use crate::comp::{GameEntity, Health, Mana};
use crate::persistence::Persistable;
use crate::sync::Reset;
use bevy::prelude::*;
use derive_more::{Deref, From};
use silkroad_agent_persistence::{CharacterRace as LoadedRace, ServerUser, WorldJoinCharacter};
use silkroad_game_base::{Character, Race, SpawningState, Stats};

#[derive(Component)]
pub(crate) struct Player {
    pub user: ServerUser,
    pub character: Character,
}

#[derive(Component, Deref, Copy, Clone, From)]
pub(crate) struct CharacterRace(Race);

impl CharacterRace {
    pub(crate) fn inner(&self) -> Race {
        self.0
    }
}

impl Player {
    pub fn from_loaded(user: ServerUser, loaded: &WorldJoinCharacter) -> Self {
        let race = match loaded.race {
            LoadedRace::Chinese => Race::Chinese,
            LoadedRace::European => Race::European,
        };
        let character = Character {
            id: loaded.id,
            name: loaded.name.clone(),
            race,
            scale: loaded.scale,
            level: loaded.level,
            max_level: loaded.max_level,
            exp: loaded.experience,
            sp: loaded.skill_points,
            sp_exp: loaded.skill_experience,
            stats: Stats::new_preallocated(loaded.strength, loaded.intelligence),
            stat_points: loaded.stat_points,
            current_hp: loaded.current_hp,
            current_mp: loaded.current_mp,
            berserk_points: loaded.berserk_points,
            gold: loaded.gold,
            beginner_mark: loaded.beginner_mark,
            gm: loaded.game_master,
            state: SpawningState::Loading,
            masteries: loaded.masteries.clone(),
            skills: loaded.skills.clone(),
        };
        Player { user, character }
    }
}

#[derive(Component)]
pub(crate) struct Buffed {
    // pub buffs: Vec<Buff>
}

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    player: Player,
    inventory: PlayerInventory,
    gold: GoldPouch,
    game_entity: GameEntity,
    agent: Agent,
    pos: Position,
    buff: Buffed,
    visibility: Visibility,
    state_queue: AgentStateQueue,
    speed: MovementState,
    damage_receiver: DamageReceiver,
    health: Health,
    mana: Mana,
    level: Leveled,
    sp: SP,
    exp: Experienced,
    goal: GoalTracker,
    persistence: Persistable,
    stat_points: StatPoints,
    masteries: MasteryKnowledge,
    skills: SkillBook,
    race: CharacterRace,
    hotbar: Hotbar,
}

impl PlayerBundle {
    pub fn new(
        player: Player,
        game_entity: GameEntity,
        inventory: PlayerInventory,
        gold: GoldPouch,
        agent: Agent,
        pos: Position,
        visibility: Visibility,
        hotbar: Hotbar,
    ) -> Self {
        let stat_points = StatPoints::new(player.character.stats, player.character.stat_points);
        let level = player.character.level;
        let max_hp = stat_points.stats().max_health(level);
        let max_mana = stat_points.stats().max_mana(level);
        let sp = player.character.sp;
        let sp_exp = player.character.sp_exp;
        let exp = player.character.exp;
        let max_level = player.character.max_level;
        let master_knowledge = MasteryKnowledge::new(&player.character.masteries);
        let skills = SkillBook::new(&player.character.skills);
        let race = player.character.race.into();
        Self {
            player,
            game_entity,
            inventory,
            agent,
            pos,
            buff: Buffed {},
            visibility,
            gold,
            state_queue: Default::default(),
            speed: MovementState::default_player(),
            damage_receiver: DamageReceiver::default(),
            health: Health::new(max_hp),
            mana: Mana::with_max(max_mana),
            sp: SP::new(sp),
            level: Leveled::new(level, max_level),
            exp: Experienced::new(exp, sp_exp as u64),
            goal: GoalTracker::default(),
            persistence: Persistable,
            stat_points,
            masteries: master_knowledge,
            skills,
            race,
            hotbar,
        }
    }
}

#[derive(Component)]
pub(crate) struct StatPoints {
    stats: Stats,
    remaining_points: u16,
    has_gained_points: bool,
    has_spent_points: bool,
}

impl StatPoints {
    pub(crate) fn new(stats: Stats, remaining_points: u16) -> Self {
        StatPoints {
            stats,
            remaining_points,
            has_gained_points: false,
            has_spent_points: false,
        }
    }

    pub(crate) fn stats(&self) -> Stats {
        self.stats
    }

    pub(crate) fn remaining_points(&self) -> u16 {
        self.remaining_points
    }

    pub(crate) fn spend_str(&mut self) {
        self.spend_str_points(1);
    }

    pub(crate) fn spend_str_points(&mut self, points: u16) {
        if self.remaining_points < points {
            return;
        }

        self.stats.increase_strength(points);
        self.remaining_points -= points;
        self.has_spent_points = true;
    }

    pub(crate) fn spend_int(&mut self) {
        self.spend_int_points(1);
    }

    pub(crate) fn spend_int_points(&mut self, points: u16) {
        if self.remaining_points < points {
            return;
        }

        self.stats.increase_intelligence(points);
        self.remaining_points -= points;
        self.has_spent_points = true;
    }

    pub(crate) fn gain_points(&mut self, amount: u16) {
        self.remaining_points = self.remaining_points.saturating_add(amount);
        self.has_gained_points = true;
    }

    pub(crate) fn has_spent_points(&self) -> bool {
        self.has_spent_points
    }

    pub(crate) fn has_gained_points(&self) -> bool {
        self.has_gained_points
    }
}

impl Reset for StatPoints {
    fn reset(&mut self) {
        self.has_gained_points = false;
        self.has_spent_points = true;
    }
}
