use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Ord, PartialOrd)]
pub struct TypeId(pub u8, pub u8, pub u8, pub u8);

impl Display for TypeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}|{}|{}|{}", self.0, self.1, self.2, self.3)
    }
}

#[derive(Copy, Clone)]
pub enum ObjectType {
    Entity(ObjectEntity),
    Item(ObjectItem),
    Area(ObjectArea),
}

impl ObjectType {
    pub fn type_id(&self) -> TypeId {
        match self {
            ObjectType::Entity(ref entity) => {
                let (t1, t2, t3) = entity.type_value();
                TypeId(1, t1, t2, t3)
            },
            ObjectType::Item(ref item) => {
                let (t1, t2, t3) = item.type_value();
                TypeId(3, t1, t2, t3)
            },
            ObjectType::Area(_) => TypeId(4, 0, 0, 0),
        }
    }

    pub fn from_type_id(type_id: &TypeId) -> Option<ObjectType> {
        match type_id.0 {
            1 => Some(ObjectType::Entity(ObjectEntity::from_type_value(
                type_id.1, type_id.2, type_id.3,
            )?)),
            3 => Some(ObjectType::Item(ObjectItem::from_type_value(
                type_id.1, type_id.2, type_id.3,
            )?)),
            4 => Some(ObjectType::Area(ObjectArea::None)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObjectEntity {
    Player,
    NonPlayer(ObjectNonPlayer),
}

impl ObjectEntity {
    pub fn type_value(&self) -> (u8, u8, u8) {
        match self {
            ObjectEntity::Player => (1, 0, 0),
            ObjectEntity::NonPlayer(ref non_player) => {
                let (t2, t3) = non_player.type_value();
                (2, t2, t3)
            },
        }
    }

    pub fn from_type_value(t1: u8, t2: u8, t3: u8) -> Option<ObjectEntity> {
        match t1 {
            1 => Some(ObjectEntity::Player),
            2 => Some(ObjectEntity::NonPlayer(ObjectNonPlayer::from_type_value(t2, t3)?)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObjectNonPlayer {
    Monster(ObjectMonster),
    NPC(ObjectNpc),
    Pet(ObjectPet),
    FortressWar(ObjectFortressWar),
    FortificationStructure(ObjectFortificationStructure),
    HumanDefenseTower,
    TreasureBox,
    Safezone,
}

impl ObjectNonPlayer {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectNonPlayer::Monster(monster) => (1, monster.into()),
            ObjectNonPlayer::NPC(npc) => (2, npc.into()),
            ObjectNonPlayer::Pet(pet) => (3, pet.into()),
            ObjectNonPlayer::FortressWar(fw) => (4, fw.into()),
            ObjectNonPlayer::FortificationStructure(structure) => (5, structure.into()),
            ObjectNonPlayer::HumanDefenseTower => (6, 0),
            ObjectNonPlayer::TreasureBox => (7, 0),
            ObjectNonPlayer::Safezone => (8, 0),
        }
    }

    pub fn from_type_value(t2: u8, t3: u8) -> Option<ObjectNonPlayer> {
        match t2 {
            1 => Some(ObjectNonPlayer::Monster(ObjectMonster::try_from(t3).ok()?)),
            2 => Some(ObjectNonPlayer::NPC(ObjectNpc::try_from(t3).ok()?)),
            3 => Some(ObjectNonPlayer::Pet(ObjectPet::try_from(t3).ok()?)),
            4 => Some(ObjectNonPlayer::FortressWar(ObjectFortressWar::try_from(t3).ok()?)),
            5 => Some(ObjectNonPlayer::FortificationStructure(
                ObjectFortificationStructure::try_from(t3).ok()?,
            )),
            6 => Some(ObjectNonPlayer::HumanDefenseTower),
            7 => Some(ObjectNonPlayer::TreasureBox),
            8 => Some(ObjectNonPlayer::Safezone),
            _ => None,
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectMonster {
    General = 1,
    Thief,
    Hunter,
    Quest,
    PandorasBox,
    Caravan,
    ThiefCaravan,
    FlowerStructure,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectNpc {
    Standard = 0,
    GateStructure,
    DungeonSlave,
    DungeonStone,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectPet {
    Vehicle = 1,
    TradeVehicle,
    AttackPet,
    AbilityPet,
    GuildSoldier,
    QuestCOS,
    QuestNPCCOS,
    Unknown,
    AttackVehiclePet,
    BatteringRam,
    Catapult,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectFortressWar {
    Guard = 1,
    Flag,
    Monster,
    OtherGuard,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectFortificationStructure {
    FortStone = 1,
    Tower,
    Gate,
    DefensivePosition,
    Headquarters,
    Barricade,
}

#[derive(Copy, Clone)]
pub enum ObjectItem {
    Equippable(ObjectEquippable),
    Summon(ObjectSummon),
    Consumable(ObjectConsumable),
    Trade(ObjectItemTrade),
    Pet(ObjectItemPet),
}

impl ObjectItem {
    pub fn type_value(&self) -> (u8, u8, u8) {
        match self {
            ObjectItem::Equippable(ref equipment) => {
                let (t2, t3) = equipment.type_value();
                (1, t2, t3)
            },
            ObjectItem::Summon(ref summon) => {
                let (t2, t3) = summon.type_value();
                (2, t2, t3)
            },
            ObjectItem::Consumable(ref consumable) => {
                let (t2, t3) = consumable.type_value();
                (3, t2, t3)
            },
            ObjectItem::Trade(ref trade) => {
                let (t2, t3) = trade.type_value();
                (4, t2, t3)
            },
            ObjectItem::Pet(ref pet) => {
                let (t2, t3) = pet.type_value();
                (5, t2, t3)
            },
        }
    }

    pub fn from_type_value(t1: u8, t2: u8, t3: u8) -> Option<ObjectItem> {
        match t1 {
            1 => Some(ObjectItem::Equippable(ObjectEquippable::from_type_value(t2, t3)?)),
            2 => Some(ObjectItem::Summon(ObjectSummon::from_type_value(t2, t3)?)),
            3 => Some(ObjectItem::Consumable(ObjectConsumable::from_type_value(t2, t3)?)),
            4 => Some(ObjectItem::Trade(ObjectItemTrade::from_type_value(t2, t3)?)),
            5 => Some(ObjectItem::Pet(ObjectItemPet::from_type_value(t2, t3)?)),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObjectEquippable {
    Clothing(ObjectClothingType, ObjectClothingPart),
    Shield(ObjectRace),
    Jewelry(ObjectRace, ObjectJewelryType),
    Weapon(ObjectWeaponType),
    Flag(ObjectFlagType),
    TradeGoods(ObjectTradeGoods),
    Avatar(ObjectAvatar),
    DevilSpirit,
}

impl ObjectEquippable {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectEquippable::Clothing(clothing, part) => (clothing.into(), part.into()),
            ObjectEquippable::Shield(race) => (4, race.into()),
            ObjectEquippable::Jewelry(race, jewelry) => {
                let race = if race == ObjectRace::Chinese { 5 } else { 12 };
                (race, jewelry.into())
            },
            ObjectEquippable::Weapon(weapon) => (6, weapon.into()),
            ObjectEquippable::Flag(flag) => (7, flag.into()),
            ObjectEquippable::TradeGoods(goods) => (8, goods.into()),
            ObjectEquippable::Avatar(avatar) => (13, avatar.into()),
            ObjectEquippable::DevilSpirit => (14, 0),
        }
    }

    pub fn from_type_value(t2: u8, t3: u8) -> Option<ObjectEquippable> {
        match t2 {
            4 => Some(ObjectEquippable::Shield(ObjectRace::try_from(t3).ok()?)),
            5 => Some(ObjectEquippable::Jewelry(
                ObjectRace::Chinese,
                ObjectJewelryType::try_from(t3).ok()?,
            )),
            6 => Some(ObjectEquippable::Weapon(ObjectWeaponType::try_from(t3).ok()?)),
            7 => Some(ObjectEquippable::Flag(ObjectFlagType::try_from(t3).ok()?)),
            8 => Some(ObjectEquippable::TradeGoods(ObjectTradeGoods::try_from(t3).ok()?)),
            12 => Some(ObjectEquippable::Jewelry(
                ObjectRace::European,
                ObjectJewelryType::try_from(t3).ok()?,
            )),
            13 => Some(ObjectEquippable::Avatar(ObjectAvatar::try_from(t3).ok()?)),
            14 => Some(ObjectEquippable::DevilSpirit),
            id if id <= 11 => Some(ObjectEquippable::Clothing(
                ObjectClothingType::try_from(id).ok()?,
                ObjectClothingPart::try_from(t3).ok()?,
            )),
            _ => None,
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectClothingType {
    Garment = 1,
    Protector = 2,
    Armor = 3,
    Robe = 9,
    LightArmor = 10,
    HeavyArmor = 11,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectClothingPart {
    Any = 0,
    Head,
    Shoulder,
    Body,
    Leg,
    Arm,
    Foot,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectRace {
    Chinese = 1,
    European,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectJewelryType {
    Earring = 1,
    Necklace,
    Ring,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectWeaponType {
    None = 1,
    Sword,
    Blade,
    Spear,
    Glavie,
    Bow,
    OneHandSword,
    TwoHandSword,
    Axe,
    WarlockStaff,
    Staff,
    Crossbow,
    Dagger,
    Harp,
    ClericRod,
    FortressHammer,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectFlagType {
    OldTrader = 1,
    OldThief,
    OldHunter,
    PvpSuit = 5,
    NewTrader,
    NewHunter,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectTradeGoods {
    Goods = 1,
    SpecialBox = 3,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectAvatar {
    Dress = 1,
    Hat,
    Attachment,
    ETC,
}

#[derive(Copy, Clone)]
pub enum ObjectSummon {
    Scroll(ObjectSummonScroll),
    Transfer,
    PreMall,
}

impl ObjectSummon {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectSummon::Scroll(scroll) => (1, scroll.into()),
            ObjectSummon::Transfer => (2, 0),
            ObjectSummon::PreMall => (3, 1),
        }
    }

    pub fn from_type_value(t2: u8, t3: u8) -> Option<ObjectSummon> {
        match t2 {
            1 => Some(ObjectSummon::Scroll(ObjectSummonScroll::try_from(t3).ok()?)),
            2 => Some(ObjectSummon::Transfer),
            3 => Some(ObjectSummon::PreMall),
            _ => None,
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectSummonScroll {
    Attack = 1,
    Ability,
    AttackAbility,
    Trade,
}

#[derive(Copy, Clone)]
pub enum ObjectConsumable {
    Recovery(ObjectConsumableRecovery),
    Cure(ObjectConsumableCure),
    Scroll(ObjectConsumableScroll),
    Ammo(ObjectConsumableAmmo),
    Currency(ObjectConsumableCurrency),
    Throwable(ObjectConsumableThrowable),
    Campfire,
    Trade(ObjectConsumableTrade),
    Quest(ObjectConsumableQuest),
    AlchemyUpgrade,
    AlchemyImprovement,
    Event(ObjectConsumableEvent),
    ItemMall(ObjectConsumableItemMall),
    MagicPop(ObjectConsumableMagicPop),
    MonsterSummon,
    Pet(ObjectConsumablePet),
}

impl ObjectConsumable {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectConsumable::Recovery(recovery) => (1, recovery.into()),
            ObjectConsumable::Cure(cure) => (2, cure.into()),
            ObjectConsumable::Scroll(scroll) => (3, scroll.into()),
            ObjectConsumable::Ammo(ammo) => (4, ammo.into()),
            ObjectConsumable::Currency(currency) => (5, currency.into()),
            ObjectConsumable::Throwable(throwable) => (6, throwable.into()),
            ObjectConsumable::Campfire => (7, 1),
            ObjectConsumable::Trade(trade) => (8, trade.into()),
            ObjectConsumable::Quest(quest) => (9, quest.into()),
            ObjectConsumable::AlchemyUpgrade => (10, 1),
            ObjectConsumable::AlchemyImprovement => (11, 1),
            ObjectConsumable::Event(event) => (12, event.into()),
            ObjectConsumable::ItemMall(mall) => (13, mall.into()),
            ObjectConsumable::MagicPop(pop) => (14, pop.into()),
            ObjectConsumable::MonsterSummon => (15, 1),
            ObjectConsumable::Pet(pet) => (16, pet.into()),
        }
    }

    pub fn from_type_value(t2: u8, t3: u8) -> Option<ObjectConsumable> {
        match t2 {
            1 => Some(ObjectConsumable::Recovery(ObjectConsumableRecovery::try_from(t3).ok()?)),
            2 => Some(ObjectConsumable::Cure(ObjectConsumableCure::try_from(t3).ok()?)),
            3 => Some(ObjectConsumable::Scroll(ObjectConsumableScroll::try_from(t3).ok()?)),
            4 => Some(ObjectConsumable::Ammo(ObjectConsumableAmmo::try_from(t3).ok()?)),
            5 => Some(ObjectConsumable::Currency(ObjectConsumableCurrency::try_from(t3).ok()?)),
            6 => Some(ObjectConsumable::Throwable(
                ObjectConsumableThrowable::try_from(t3).ok()?,
            )),
            7 => Some(ObjectConsumable::Campfire),
            8 => Some(ObjectConsumable::Trade(ObjectConsumableTrade::try_from(t3).ok()?)),
            9 => Some(ObjectConsumable::Quest(ObjectConsumableQuest::try_from(t3).ok()?)),
            10 => Some(ObjectConsumable::AlchemyUpgrade),
            11 => Some(ObjectConsumable::AlchemyImprovement),
            12 => Some(ObjectConsumable::Event(ObjectConsumableEvent::try_from(t3).ok()?)),
            13 => Some(ObjectConsumable::ItemMall(ObjectConsumableItemMall::try_from(t3).ok()?)),
            14 => Some(ObjectConsumable::MagicPop(ObjectConsumableMagicPop::try_from(t3).ok()?)),
            15 => Some(ObjectConsumable::MonsterSummon),
            16 => Some(ObjectConsumable::Pet(ObjectConsumablePet::try_from(t3).ok()?)),
            _ => None,
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableRecovery {
    HP = 1,
    MP,
    Vigor,
    CosHP,
    CosRevive = 6,
    Berserk = 8,
    HGP,
    Repair,
    PreMall,
    PetHunger,
    Unknown,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableCure {
    Single = 1,
    Full = 6,
    CosFull,
    Superset,
    Super,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableScroll {
    Return = 1,
    Transport,
    Reverse,
    MonsterSummon,
    GlobalChat,
    FortTablet,
    GuardSummon,
    FortManual,
    FortressWarFlag,
    Buff,
    FortressWarGuardian,
    SkillPoint,
    ServerChange,
    ItemUpgrade,
    PremiumQuest,
    Coupon,
    NasrunUpgrade,
    ExpStone,
    UnionPartyTicket,
    FlowerSummon,
    AlchemyUp,
    GlobalChat2,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableAmmo {
    Arrows = 1,
    Bolts,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableCurrency {
    Gold = 0,
    Token,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableThrowable {
    Firework = 1,
    Shock,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableTrade {
    General = 0,
    Lower,
    // These names make no sense
    Higher,
    SpecialBox,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableQuest {
    General = 0,
    Job,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableEvent {
    GuildSoldier = 1,
    GuildCrest,
    GuildRecall,
    BattleArena,
    Talisman,
    ChangeTicket,
    ForgottenWorld,
    ChargeGold,
    Survival,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableItemMall {
    SkillRestore = 0,
    Buff,
    ExpVIP = 4,
    SPVIP,
    Resurrection,
    Repair,
    GenderChange,
    Transmog,
    Warehouse,
    ReinforceWeapon,
    PetTimeExtension,
    StatPointsReset,
    Premium,
    PetGrowth,
    Nasrun,
    InventoryExtension,
    Indulgence,
    OpenMarket,
    FreeAccess,
    Exp,
    Dungeon,
    ForgottenWorld,
    SafetyDevice,
    FreeTime,
    SurvivalDetection,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumableMagicPop {
    EntryCard = 1,
    Result,
    Pre,
    Pre2,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectConsumablePet {
    General = 1,
    NameChange = 3,
    Level,
    Evolution,
}

#[derive(Copy, Clone)]
pub enum ObjectItemTrade {
    HunterArmor(ObjectItemTradeArmor),
    HunterWeapon,
    HunterJewelry(ObjectItemTradeJewelry),
    ThiefArmor(ObjectItemTradeArmor),
    ThiefWeapon,
    ThiefJewelry(ObjectItemTradeJewelry),
}

impl ObjectItemTrade {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectItemTrade::HunterArmor(armor) => (1, armor.into()),
            ObjectItemTrade::HunterWeapon => (2, 1),
            ObjectItemTrade::HunterJewelry(jewelry) => (3, jewelry.into()),
            ObjectItemTrade::ThiefArmor(armor) => (4, armor.into()),
            ObjectItemTrade::ThiefWeapon => (5, 1),
            ObjectItemTrade::ThiefJewelry(jewelry) => (6, jewelry.into()),
        }
    }

    pub fn from_type_value(t2: u8, t3: u8) -> Option<ObjectItemTrade> {
        match t2 {
            1 => Some(ObjectItemTrade::HunterArmor(ObjectItemTradeArmor::try_from(t3).ok()?)),
            2 => Some(ObjectItemTrade::HunterWeapon),
            3 => Some(ObjectItemTrade::HunterJewelry(
                ObjectItemTradeJewelry::try_from(t3).ok()?,
            )),
            4 => Some(ObjectItemTrade::ThiefArmor(ObjectItemTradeArmor::try_from(t3).ok()?)),
            5 => Some(ObjectItemTrade::ThiefWeapon),
            6 => Some(ObjectItemTrade::ThiefJewelry(
                ObjectItemTradeJewelry::try_from(t3).ok()?,
            )),
            _ => None,
        }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectItemTradeArmor {
    Head = 1,
    Shoulder,
    Chest,
    Pants,
    Gloves,
    Shoes,
}

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum ObjectItemTradeJewelry {
    Earring = 1,
    Necklace,
    Ring,
}

#[derive(Copy, Clone)]
pub enum ObjectItemPet {
    Claw,
    Charm,
    Saddle,
    Scale,
    Amulet,
    Choker,
    Tattoo,
}

impl ObjectItemPet {
    pub fn type_value(self) -> (u8, u8) {
        match self {
            ObjectItemPet::Claw => (1, 0),
            ObjectItemPet::Charm => (2, 0),
            ObjectItemPet::Saddle => (3, 0),
            ObjectItemPet::Scale => (4, 0),
            ObjectItemPet::Amulet => (5, 0),
            ObjectItemPet::Choker => (6, 0),
            ObjectItemPet::Tattoo => (7, 0),
        }
    }

    pub fn from_type_value(t2: u8, _t3: u8) -> Option<ObjectItemPet> {
        match t2 {
            1 => Some(ObjectItemPet::Claw),
            2 => Some(ObjectItemPet::Charm),
            3 => Some(ObjectItemPet::Saddle),
            4 => Some(ObjectItemPet::Scale),
            5 => Some(ObjectItemPet::Amulet),
            6 => Some(ObjectItemPet::Choker),
            7 => Some(ObjectItemPet::Tattoo),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialOrd, PartialEq)]
pub enum ObjectArea {
    None,
}
