use once_cell::sync::OnceCell;
use pk2::Pk2;
use silkroad_data::characterdata::{load_character_map, RefCharacterData};
use silkroad_data::datamap::DataMap;
use silkroad_data::gold::{load_gold_map, GoldMap};
use silkroad_data::itemdata::{load_item_map, RefItemData};
use silkroad_data::level::{load_level_map, LevelMap};
use silkroad_data::skilldata::{load_skill_map, RefSkillData};
use silkroad_data::FileError;

static ITEMS: OnceCell<DataMap<RefItemData>> = OnceCell::new();
static CHARACTERS: OnceCell<DataMap<RefCharacterData>> = OnceCell::new();
static SKILLS: OnceCell<DataMap<RefSkillData>> = OnceCell::new();
static LEVELS: OnceCell<LevelMap> = OnceCell::new();
static GOLD: OnceCell<GoldMap> = OnceCell::new();

pub struct WorldData;

impl WorldData {
    pub(crate) fn load_data_from(media_pk2: &Pk2) -> Result<(), FileError> {
        let levels = load_level_map(&media_pk2)?;
        let gold = load_gold_map(&media_pk2)?;
        let characters = load_character_map(&media_pk2)?;
        let items = load_item_map(&media_pk2)?;
        let skills = load_skill_map(&media_pk2)?;
        let _ = LEVELS.set(levels);
        let _ = GOLD.set(gold);
        let _ = CHARACTERS.set(characters);
        let _ = ITEMS.set(items);
        let _ = SKILLS.set(skills);
        Ok(())
    }

    pub fn items() -> &'static DataMap<RefItemData> {
        ITEMS.get().expect("Items should have been set")
    }

    pub fn characters() -> &'static DataMap<RefCharacterData> {
        CHARACTERS.get().expect("Characters should have been set")
    }

    pub fn skills() -> &'static DataMap<RefSkillData> {
        SKILLS.get().expect("Skills should have been set")
    }

    pub fn levels() -> &'static LevelMap {
        LEVELS.get().expect("Levels should have been set")
    }

    pub fn gold() -> &'static GoldMap {
        GOLD.get().expect("Gold should have been set")
    }
}
