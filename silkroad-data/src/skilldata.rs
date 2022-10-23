use crate::type_id::{ObjectConsumable, ObjectEquippable, ObjectWeaponType};
use crate::{DataEntry, DataMap, FileError, ParseError};
use bitflags::bitflags;
use num_enum::TryFromPrimitive;
use pk2::Pk2;
use std::num::{NonZeroU16, NonZeroU32, NonZeroU8};
use std::ops::Deref;
use std::str::FromStr;

pub fn load_skill_map(pk2: &Pk2) -> Result<DataMap<RefSkillData>, FileError> {
    DataMap::from(pk2, "/server_dep/silkroad/textdata/SkillData.txt")
}

bitflags! {
    pub struct TargetOption: u8 {
        const NONE = 0;
        const SELF = 0b00000001;
        const ALLY = 0b00000010;
        const PARTY = 0b00000100;
        const ENEMY_MONSTER = 0b00001000;
        const ENEMY_PLAYER = 0b00010000;
        const NETRAL = 0b00100000;
        const ANY = 0b01000000;
        const DEAD = 0b10000000;
    }
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone)]
pub enum SkillType {
    Passive = 0,
    Imbue = 1,
    Action = 2,
}

impl FromStr for SkillType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(SkillType::try_from(value)?)
    }
}

#[repr(u8)]
#[derive(TryFromPrimitive, Copy, Clone)]
pub enum AutoAttack {
    No = 0,
    Yes = 1,
    Range = 2, // or 'special'? like on ground
}

impl FromStr for AutoAttack {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value: u8 = s.parse()?;
        Ok(AutoAttack::try_from(value)?)
    }
}

pub struct LearnedSkill {
    ref_id: u32,
    level: u8,
}

pub struct SkillTimings {
    pub preparation_time: u32,
    pub cast_time: u32,
    pub duration: i32,
    // can be negative to apply to original?
    pub cooldown: u32,
    pub next_delay: u32,
}

pub struct RefSkillData {
    pub ref_id: u32,
    pub group: u32,
    pub id: String,
    pub original: Option<NonZeroU32>,
    pub level: u8,
    pub type_: SkillType,
    pub next_in_chain: Option<NonZeroU32>,
    pub timings: SkillTimings,
    pub projectile_speed: u32,
    // Bitflags with created buffs that might interfere with another
    pub buff_interference: u32,
    pub auto_attack: AutoAttack,
    pub range: u16,
    pub requires_target: bool,
    pub target: TargetOption,
    pub mastery: Option<NonZeroU16>,
    pub mastery_level: Option<NonZeroU8>,
    pub required_skills: Vec<LearnedSkill>,
    pub sp: u32,
    pub race: u8,
    pub weapon_requirements: [Option<ObjectWeaponType>; 2],
    pub consumed_hp: u32,
    pub consumed_mp: u32,
    pub usage_chance: u8,
    pub usage_type: u8,
    pub params: Vec<SkillParam>,
}

impl DataEntry for RefSkillData {
    fn ref_id(&self) -> u32 {
        self.ref_id
    }

    fn code(&self) -> &str {
        &self.id
    }
}

impl FromStr for RefSkillData {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements = s.split('\t').collect::<Vec<&str>>();
        let next: u32 = elements.get(9).ok_or(ParseError::MissingColumn(9))?.parse()?;
        let original: u32 = elements.get(6).ok_or(ParseError::MissingColumn(6))?.parse()?;
        let required_target: u8 = elements.get(22).ok_or(ParseError::MissingColumn(22))?.parse()?;

        let target_self: u8 = elements.get(26).ok_or(ParseError::MissingColumn(26))?.parse()?;
        let target_ally: u8 = elements.get(27).ok_or(ParseError::MissingColumn(27))?.parse()?;
        let target_party: u8 = elements.get(28).ok_or(ParseError::MissingColumn(28))?.parse()?;
        let target_enemy_monster: u8 = elements.get(29).ok_or(ParseError::MissingColumn(29))?.parse()?;
        let target_enemy_player: u8 = elements.get(30).ok_or(ParseError::MissingColumn(30))?.parse()?;
        let target_neutral: u8 = elements.get(31).ok_or(ParseError::MissingColumn(31))?.parse()?;
        let target_any: u8 = elements.get(32).ok_or(ParseError::MissingColumn(32))?.parse()?;
        let target_dead: u8 = elements.get(33).ok_or(ParseError::MissingColumn(33))?.parse()?;
        let mut target_options = TargetOption::NONE;
        if target_self != 0 {
            target_options |= TargetOption::SELF;
        }

        if target_ally != 0 {
            target_options |= TargetOption::ALLY;
        }

        if target_party != 0 {
            target_options |= TargetOption::PARTY;
        }

        if target_enemy_monster != 0 {
            target_options |= TargetOption::ENEMY_MONSTER;
        }

        if target_enemy_player != 0 {
            target_options |= TargetOption::ENEMY_PLAYER;
        }

        if target_neutral != 0 {
            target_options |= TargetOption::NETRAL;
        }

        if target_any != 0 {
            target_options |= TargetOption::ANY;
        }

        if target_dead != 0 {
            target_options |= TargetOption::DEAD;
        }

        let mastery: u16 = elements.get(34).ok_or(ParseError::MissingColumn(34))?.parse()?;
        let mastery_level: u8 = elements.get(36).ok_or(ParseError::MissingColumn(36))?.parse()?;
        let skills = vec![
            LearnedSkill {
                ref_id: elements.get(40).ok_or(ParseError::MissingColumn(40))?.parse()?,
                level: elements.get(43).ok_or(ParseError::MissingColumn(43))?.parse()?,
            },
            LearnedSkill {
                ref_id: elements.get(41).ok_or(ParseError::MissingColumn(41))?.parse()?,
                level: elements.get(44).ok_or(ParseError::MissingColumn(44))?.parse()?,
            },
            LearnedSkill {
                ref_id: elements.get(42).ok_or(ParseError::MissingColumn(42))?.parse()?,
                level: elements.get(45).ok_or(ParseError::MissingColumn(45))?.parse()?,
            },
        ]
        .into_iter()
        .filter(|skill| skill.ref_id != 0)
        .collect::<Vec<_>>();

        let mut params = Vec::new();
        let number_column = elements[69..]
            .iter()
            .map(|col| {
                let val: i32 = col.parse().unwrap();
                val as u32
            })
            .collect::<Vec<u32>>();
        let mut remaining = number_column.deref();
        while let Some((next, param)) = parse_param(remaining) {
            params.push(param);
            remaining = next;
        }

        let first_weapon_requirement: u8 = elements.get(50).ok_or(ParseError::MissingColumn(50))?.parse()?;
        let second_weapon_requirement: u8 = elements.get(51).ok_or(ParseError::MissingColumn(51))?.parse()?;
        let buff_inference: i32 = elements.get(18).ok_or(ParseError::MissingColumn(18))?.parse()?;
        Ok(Self {
            ref_id: elements.get(1).ok_or(ParseError::MissingColumn(1))?.parse()?,
            group: elements.get(2).ok_or(ParseError::MissingColumn(2))?.parse()?,
            id: elements.get(3).ok_or(ParseError::MissingColumn(3))?.to_string(),
            original: NonZeroU32::new(original),
            level: elements.get(7).ok_or(ParseError::MissingColumn(7))?.parse()?,
            type_: elements.get(8).ok_or(ParseError::MissingColumn(8))?.parse()?,
            next_in_chain: NonZeroU32::new(next),
            timings: SkillTimings {
                preparation_time: elements.get(11).ok_or(ParseError::MissingColumn(11))?.parse()?,
                cast_time: elements.get(12).ok_or(ParseError::MissingColumn(12))?.parse()?,
                duration: elements.get(13).ok_or(ParseError::MissingColumn(13))?.parse()?,
                cooldown: elements.get(14).ok_or(ParseError::MissingColumn(14))?.parse()?,
                next_delay: elements.get(15).ok_or(ParseError::MissingColumn(15))?.parse()?,
            },
            projectile_speed: elements.get(16).ok_or(ParseError::MissingColumn(16))?.parse()?,
            buff_interference: buff_inference as u32,
            auto_attack: elements.get(19).ok_or(ParseError::MissingColumn(19))?.parse()?,
            range: elements.get(21).ok_or(ParseError::MissingColumn(21))?.parse()?,
            requires_target: required_target != 0,
            target: target_options,
            mastery: NonZeroU16::new(mastery),
            mastery_level: NonZeroU8::new(mastery_level),
            required_skills: skills,
            sp: elements.get(46).ok_or(ParseError::MissingColumn(46))?.parse()?,
            race: elements.get(47).ok_or(ParseError::MissingColumn(47))?.parse()?,
            weapon_requirements: [
                ObjectWeaponType::try_from(first_weapon_requirement).ok(),
                ObjectWeaponType::try_from(second_weapon_requirement).ok(),
            ],
            consumed_hp: elements.get(52).ok_or(ParseError::MissingColumn(52))?.parse()?,
            consumed_mp: elements.get(53).ok_or(ParseError::MissingColumn(53))?.parse()?,
            usage_chance: elements.get(66).ok_or(ParseError::MissingColumn(66))?.parse()?,
            usage_type: elements.get(67).ok_or(ParseError::MissingColumn(67))?.parse()?,
            params,
        })
    }
}

pub enum SkillParam {
    Attack {
        kind: u32,
        phys: u32,
        min: u32,
        max: u32,
        mag: u32,
    },
    Chain {
        count: u32,
        unknown: u32,
    },
    GetValue(String),
    SetValue {
        var: String,
        value: u32,
        value_2: u32,
    },
    Duration(u32),
    RequiredItem(ObjectEquippable),
    IncreaseDefense {
        phys: u32,
        mag: u32,
        unknown: u32,
    },
    BlockRatio {
        unknown: u8,
        percent: u8,
    },
    Knockdown {
        unknown: u8,
        chance: u8,
    },
    AOE {
        origin: u8,
        // player or target?
        area_type: u8,
        // Something like cone, circle, etc.?
        area_size: u16,
        unknown_type: u8,
        val1: u32,
        // these relate to unknown_type
        val2: u32,
    },
    DownAttack(u16),
    RequiredState(u8),
    // Bitflags, 0x1 => knockdown, 0x40 => ?, 0x80 => ?
    Stun {
        duration: u32,
        chance: u8,
        level: u8,
    },
    Knockback {
        chance: u8,
        distance: u16,
    },
    IncreaseHP {
        absolute: u32,
        percent: u8,
    },
    IncreaseMP {
        absolute: u32,
        percent: u8,
    },
    ConsumeItem {
        kind: ObjectConsumable,
        amount: u8,
    },
    IncreaseCrit {
        amount: u8,
        unknown: u8, // 9, 11, 15?
    },
    IncreaseHitRate {
        attack_rate: u8,
        hit_rate: u8,
    },
    SummonBird {
        ref_id: u32,
        unknown: u8,
        // always 10
        unknown_2: u16,
        // always 3500,
        duration: u16, // ?
    },
    IncreaseRange(u16),
    Freeze {
        unknown: u16,
        chance: u8,
    },
    Frostbite {
        unknown: u16,
        chance: u8,
    },
    TantUnkn {
        unknown: u32,
        unknown_2: u32,
    },
    ConsumeMana {
        duration: u32,
        amount: u32,
    },
    CreateWall {
        unknown: u8,
        unknown_2: u32,
        empty: u8,
        level: u8, // min or max level maybe?
    },
    EffectLightning {
        unknown: u16,
        unknown_2: u8,
        unknown_3: u8,
    },
    IncreaseDamage {
        phys: u8,
        mag: u8,
    },
    IncreaseSpeed(u8),
    // percent
    Teleport {
        duration: u32,
        distance: u16,
    },
    IncreaseEvasion {
        evasion: u8,
        parry: u8,
    },
    Burn {
        max: u32,
        chance: u8,
        min: u32,
    },
    ReduceNegativeEffects {
        kind: u8,
        amount: u8,
        empty: u8,
    },
    // Name is temporary, something with abnormal status?
    Real {
        buff_interference: u32,
        chance: u8,
        level: u8,
    },
    Heal {
        hp_abs: u32,
        hp_percentage: u8,
        mp_abs: u32,
        mp_percentage: u8,
    },
    Invisible {
        kind: u8,
        level: u8,
        unknown: u8, // always 50
    },
    IncreaseExperience {
        exp_percentage: u16,
        sp_percentage: u16,
    },
    ItemBuff,
    Interval(u32),
    Resurrect {
        max_level: u8,
        experience_restore: u8,
    },
    // temp name
    PSOG(u8),
    // temp name
    NBUF,
    Debuff,
    IncreaseTaunt {
        taunt_value: u32,
        aggro_percent: u8,
    },
    SummonMonster(Vec<MonsterSummon>),
    DullDebuff {
        duration: u32,
        chance: u8,
        level: u8,
    },
    FearDebuff {
        duration: u32,
        chance: u8,
        level: u8,
    },
    BleedDebuff {
        duration: u32,
        chance: u8,
        level: u8,
        damage: u32,
        unknown: u8,
    },
    Restrain {
        duration: u32,
        chance: u8,
        level: u8,
    },
    IncreaseBerserk(u8),
    ShortSight {
        duration: u32,
        chance: u8,
        level: u8,
        distance: u16,
    },
    Poison {
        unknown: u16,
        chance: u8,
        damage: u32,
    },
    Linked {
        level: u8,
        // unclear
        distance: Option<NonZeroU32>,
        max_links: u8,
        damage_share: bool,
    },
    // temp name
    ATCA {
        unknown: u16,
        // always 16832
        unknown_2: u8, // always 50
    },
    // temp name
    MaxLinks,
    PetBuff {
        attribute: String,
        unknown: u8,
        unknown_2: u8,
        unknown_3: u8,
    },
    Impotent {
        duration: u32,
        chance: u8,
        level: u8,
        unknown: u8,
    },
    Darkness {
        duration: u32,
        chance: u8,
        level: u8,
        unknown: u8,
    },
    DecreaseMagicalDefense {
        duration: u32,
        chance: u8,
        level: u8,
        value: u16,
    },
    DecreasePhysicalDefense {
        duration: u32,
        chance: u8,
        level: u8,
        value: u16,
    },
    AbsorbDamage {
        level: u8,
        percent: u8,
    },
    Disease {
        duration: u32,
        chance: u8,
        level: u8,
        unknown: u8,
    },
    Division {
        duration: u32,
        chance: u8,
        level: u8,
        unknown: u8,
    },
    Panic {
        duration: u32,
        chance: u8,
        level: u8,
        min: u8,
        // temp
        max: u8,
        // temp
        unknown: u8,
    },
    MWHH(u16),
    // temp name
    IncreaseAttackPower {
        physical: u16,
        magical: u16,
    },
    IncreaseInt {
        amount: u8,
        unknown: u8,
    },
    IncreaseStr {
        amount: u8,
        unknown: u8,
    },
    DefaultAttack(u8),
    VIPBuff,
    Sleep {
        duration: u32,
        chance: u8,
        level: u8,
    },
}

pub struct MonsterSummon {
    ref_id: u32,
    rarity: u8,
    min: u8,
    max: u8,
}

fn parse_param(params: &[u32]) -> Option<(&[u32], SkillParam)> {
    if params.is_empty() || params[0] == 0 {
        return None;
    }

    let kind = String::from_utf8(params[0].to_be_bytes().into_iter().filter(|c| *c != 0).collect()).unwrap();
    let data = &params[1..];
    match kind.as_str() {
        "att" => {
            let (param_data, remaining) = data.split_at(5);

            Some((
                remaining,
                SkillParam::Attack {
                    kind: param_data[0],
                    phys: param_data[1],
                    min: param_data[2],
                    max: param_data[3],
                    mag: param_data[4],
                },
            ))
        },
        "mc" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Chain {
                    count: param_data[0],
                    unknown: param_data[1],
                },
            ))
        },
        "getv" => {
            let (param_data, remaining) = data.split_at(1);
            let var = String::from_utf8(param_data[0].to_be_bytes().into_iter().filter(|c| *c != 0).collect()).unwrap();
            Some((remaining, SkillParam::GetValue(var)))
        },
        "setv" => {
            let (param_data, remaining) = data.split_at(3);
            let var = String::from_utf8(param_data[0].to_be_bytes().into_iter().filter(|c| *c != 0).collect()).unwrap();
            let result = SkillParam::SetValue {
                var,
                value: param_data[1],
                value_2: param_data[2],
            };
            if !remaining.is_empty() && remaining[0] == 0 {
                Some((&remaining[1..], result))
            } else {
                Some((remaining, result))
            }
        },
        "dura" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::Duration(param_data[0])))
        },
        "reqi" => {
            let (param_data, remaining) = data.split_at(2);
            let t2 = param_data[0];
            let t3 = param_data[1];

            Some((
                remaining,
                SkillParam::RequiredItem(ObjectEquippable::from_type_value(t2 as u8, t3 as u8).unwrap()),
            ))
        },
        "defp" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::IncreaseDefense {
                    phys: param_data[0],
                    mag: param_data[1],
                    unknown: param_data[2],
                },
            ))
        },
        "br" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::BlockRatio {
                    unknown: param_data[0] as u8,
                    percent: param_data[1] as u8,
                },
            ))
        },
        "ko" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Knockdown {
                    unknown: param_data[0] as u8,
                    chance: param_data[1] as u8,
                },
            ))
        },
        "efr" => {
            let (param_data, remaining) = data.split_at(6);
            Some((
                remaining,
                SkillParam::AOE {
                    origin: param_data[0] as u8,
                    area_type: param_data[1] as u8,
                    area_size: param_data[2] as u16,
                    unknown_type: param_data[3] as u8,
                    val1: param_data[4],
                    val2: param_data[5],
                },
            ))
        },
        "da" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::DownAttack(param_data[0] as u16)))
        },
        "reqc" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::RequiredState(param_data[0] as u8)))
        },
        "st" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Stun {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        "kb" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Knockback {
                    chance: param_data[0] as u8,
                    distance: param_data[1] as u16,
                },
            ))
        },
        "hpi" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseHP {
                    absolute: param_data[0],
                    percent: param_data[1] as u8,
                },
            ))
        },
        "mpi" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseMP {
                    absolute: param_data[0],
                    percent: param_data[1] as u8,
                },
            ))
        },
        "cnsm" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::ConsumeItem {
                    kind: ObjectConsumable::from_type_value(param_data[0] as u8, param_data[1] as u8)
                        .expect("Unknown type id"),
                    amount: param_data[2] as u8,
                },
            ))
        },
        "cr" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseCrit {
                    amount: param_data[0] as u8,
                    unknown: param_data[1] as u8,
                },
            ))
        },
        "hr" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseHitRate {
                    attack_rate: param_data[0] as u8,
                    hit_rate: param_data[1] as u8,
                },
            ))
        },
        "summ" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::SummonBird {
                    ref_id: param_data[0],
                    unknown: param_data[1] as u8,
                    unknown_2: param_data[2] as u16,
                    duration: param_data[3] as u16,
                },
            ))
        },
        "ru" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::IncreaseRange(param_data[0] as u16)))
        },
        "fz" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Freeze {
                    unknown: param_data[0] as u16,
                    chance: param_data[1] as u8,
                },
            ))
        },
        "fb" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Frostbite {
                    unknown: param_data[0] as u16,
                    chance: param_data[1] as u8,
                },
            ))
        },
        "tant" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::TantUnkn {
                    unknown: param_data[0],
                    unknown_2: param_data[1],
                },
            ))
        },
        "onff" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::ConsumeMana {
                    duration: param_data[0],
                    amount: param_data[1],
                },
            ))
        },
        "pw" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::CreateWall {
                    unknown: param_data[0] as u8,
                    unknown_2: param_data[1],
                    empty: param_data[2] as u8,
                    level: param_data[3] as u8,
                },
            ))
        },
        "es" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::EffectLightning {
                    unknown: param_data[0] as u16,
                    unknown_2: param_data[1] as u8,
                    unknown_3: param_data[2] as u8,
                },
            ))
        },
        "dru" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseDamage {
                    phys: param_data[0] as u8,
                    mag: param_data[1] as u8,
                },
            ))
        },
        "hste" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::IncreaseSpeed(param_data[0] as u8)))
        },
        "tele" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Teleport {
                    duration: param_data[0],
                    distance: param_data[1] as u16,
                },
            ))
        },
        "er" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseEvasion {
                    evasion: param_data[0] as u8,
                    parry: param_data[1] as u8,
                },
            ))
        },
        "bu" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Burn {
                    max: param_data[0],
                    chance: param_data[1] as u8,
                    min: param_data[2],
                },
            ))
        },
        "bgra" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::ReduceNegativeEffects {
                    kind: param_data[0] as u8,
                    amount: param_data[1] as u8,
                    empty: param_data[2] as u8,
                },
            ))
        },
        "real" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Real {
                    buff_interference: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        "heal" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::Heal {
                    hp_abs: param_data[0],
                    hp_percentage: param_data[1] as u8,
                    mp_abs: param_data[2],
                    mp_percentage: param_data[3] as u8,
                },
            ))
        },
        "hide" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Invisible {
                    kind: param_data[0] as u8,
                    level: param_data[1] as u8,
                    unknown: param_data[2] as u8,
                },
            ))
        },
        "expu" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseExperience {
                    exp_percentage: param_data[0] as u16,
                    sp_percentage: param_data[1] as u16,
                },
            ))
        },
        "cbuf" => Some((data, SkillParam::ItemBuff)),
        "nbuf" => Some((data, SkillParam::NBUF)),
        "bbuf" => Some((data, SkillParam::Debuff)),
        "lks2" => Some((data, SkillParam::MaxLinks)),
        "puls" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::Interval(param_data[0])))
        },
        "resu" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::Resurrect {
                    max_level: param_data[0] as u8,
                    experience_restore: param_data[1] as u8,
                },
            ))
        },
        "psog" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::PSOG(param_data[0] as u8)))
        },
        "tnt2" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseTaunt {
                    taunt_value: param_data[0],
                    aggro_percent: param_data[1] as u8,
                },
            ))
        },
        "ssou" => {
            let mut summons = Vec::new();
            let mut remaining = data;
            while remaining[0] != 0 || summons.is_empty() {
                let (current, next) = remaining.split_at(4);
                let summon = MonsterSummon {
                    ref_id: current[0],
                    rarity: current[1] as u8,
                    min: current[2] as u8,
                    max: current[3] as u8,
                };
                summons.push(summon);
                remaining = next;
            }

            Some((&remaining[1..], SkillParam::SummonMonster(summons)))
        },
        "sl" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::DullDebuff {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        "bl" => {
            let (param_data, remaining) = data.split_at(5);
            Some((
                remaining,
                SkillParam::BleedDebuff {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    damage: param_data[3],
                    unknown: param_data[4] as u8,
                },
            ))
        },
        "rt" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Restrain {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        "hwdu" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::IncreaseBerserk(param_data[0] as u8)))
        },
        "my" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::ShortSight {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    distance: param_data[3] as u16,
                },
            ))
        },
        "ps" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Poison {
                    unknown: param_data[0] as u16,
                    chance: param_data[1] as u8,
                    damage: param_data[2],
                },
            ))
        },
        "lnks" => {
            let (param_data, remaining) = data.split_at(4);

            Some((
                remaining,
                SkillParam::Linked {
                    level: param_data[0] as u8,
                    distance: NonZeroU32::new(param_data[1]),
                    max_links: param_data[2] as u8,
                    damage_share: param_data[3] == 1,
                },
            ))
        },
        "atca" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::ATCA {
                    unknown: param_data[0] as u16,
                    unknown_2: param_data[1] as u8,
                },
            ))
        },
        "pet2" => {
            let (param_data, remaining) = data.split_at(4);
            let var = String::from_utf8(param_data[0].to_be_bytes().into_iter().filter(|c| *c != 0).collect()).unwrap();
            Some((
                remaining,
                SkillParam::PetBuff {
                    attribute: var,
                    unknown: param_data[1] as u8,
                    unknown_2: param_data[2] as u8,
                    unknown_3: param_data[3] as u8,
                },
            ))
        },
        "cssr" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::Impotent {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    unknown: param_data[3] as u8,
                },
            ))
        },
        "dn" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::Darkness {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    unknown: param_data[3] as u8,
                },
            ))
        },
        "fe" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::FearDebuff {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        "csmd" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::DecreaseMagicalDefense {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    value: param_data[3] as u16,
                },
            ))
        },
        "cspd" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::DecreasePhysicalDefense {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    value: param_data[3] as u16,
                },
            ))
        },
        "odar" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::AbsorbDamage {
                    level: param_data[0] as u8,
                    percent: param_data[1] as u8,
                },
            ))
        },
        "ds" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::Disease {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    unknown: param_data[3] as u8,
                },
            ))
        },
        "cshp" => {
            let (param_data, remaining) = data.split_at(6);
            Some((
                remaining,
                SkillParam::Panic {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    min: param_data[3] as u8,
                    max: param_data[3] as u8,
                    unknown: param_data[3] as u8,
                },
            ))
        },
        "mwhh" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::MWHH(param_data[0] as u16)))
        },
        "apau" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseAttackPower {
                    physical: param_data[0] as u16,
                    magical: param_data[1] as u16,
                },
            ))
        },
        "csit" => {
            let (param_data, remaining) = data.split_at(4);
            Some((
                remaining,
                SkillParam::Division {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                    unknown: param_data[3] as u8,
                },
            ))
        },
        "inti" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseInt {
                    amount: param_data[0] as u8,
                    unknown: param_data[1] as u8,
                },
            ))
        },
        "stri" => {
            let (param_data, remaining) = data.split_at(2);
            Some((
                remaining,
                SkillParam::IncreaseStr {
                    amount: param_data[0] as u8,
                    unknown: param_data[1] as u8,
                },
            ))
        },
        "ck" => {
            let (param_data, remaining) = data.split_at(1);
            Some((remaining, SkillParam::DefaultAttack(param_data[0] as u8)))
        },
        "vbuf" => Some((data, SkillParam::VIPBuff)),
        "se" => {
            let (param_data, remaining) = data.split_at(3);
            Some((
                remaining,
                SkillParam::Sleep {
                    duration: param_data[0],
                    chance: param_data[1] as u8,
                    level: param_data[2] as u8,
                },
            ))
        },
        _ => {
            // println!("Unknown param kind: {kind}({})", params[0]);
            None
        },
    }
}
