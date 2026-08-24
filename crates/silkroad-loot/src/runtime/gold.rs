use silkroad_data::itemdata::RefItemData;
use silkroad_game_base::{Item, ItemTypeData};

/// Reference ids of the three gold visual tiers in the Media catalogue.
pub(crate) const GOLD_ITEM_SMALL_REF_ID: u32 = 1;
pub(crate) const GOLD_ITEM_MEDIUM_REF_ID: u32 = 2;
pub(crate) const GOLD_ITEM_LARGE_REF_ID: u32 = 3;

/// Amounts below this threshold are represented by the small gold item.
pub(crate) const GOLD_TIER_SMALL_MAX: u32 = 1000;
/// Amounts below this threshold are represented by the medium gold item.
pub(crate) const GOLD_TIER_MEDIUM_MAX: u32 = 5000;

/// References to the three gold visual tiers, resolved from the Media catalogue.
pub(crate) struct GoldItemReferences {
    pub(super) small: &'static RefItemData,
    pub(super) medium: &'static RefItemData,
    pub(super) large: &'static RefItemData,
}

impl GoldItemReferences {
    pub(crate) fn resolve(items: &'static silkroad_data::DataMap<RefItemData>) -> Result<Self, Vec<u32>> {
        let small = items.find_id(GOLD_ITEM_SMALL_REF_ID);
        let medium = items.find_id(GOLD_ITEM_MEDIUM_REF_ID);
        let large = items.find_id(GOLD_ITEM_LARGE_REF_ID);
        match (small, medium, large) {
            (Some(small), Some(medium), Some(large)) => Ok(Self { small, medium, large }),
            _ => {
                let missing = [
                    (GOLD_ITEM_SMALL_REF_ID, small.is_none()),
                    (GOLD_ITEM_MEDIUM_REF_ID, medium.is_none()),
                    (GOLD_ITEM_LARGE_REF_ID, large.is_none()),
                ]
                .into_iter()
                .filter_map(|(id, is_missing)| is_missing.then_some(id))
                .collect();
                Err(missing)
            },
        }
    }

    pub(crate) fn reference_for_amount(&self, amount: u32) -> &'static RefItemData {
        if amount < GOLD_TIER_SMALL_MAX {
            self.small
        } else if amount < GOLD_TIER_MEDIUM_MAX {
            self.medium
        } else {
            self.large
        }
    }

    pub(crate) fn construct_item(&self, amount: u32) -> Item {
        Item {
            reference: self.reference_for_amount(amount),
            variance: None,
            type_data: ItemTypeData::Gold { amount },
        }
    }
}
