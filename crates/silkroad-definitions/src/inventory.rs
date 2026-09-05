use num_enum_derive::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, Hash, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum EquipmentSlot {
    HeadArmor = 0,
    ShoulderArmor = 1,
    ChestArmor = 2,
    WristArmor = 3,
    LegArmor = 4,
    FootArmor = 5,
    Weapon = 6,
    SecondaryWeapon = 7,
    Earring = 8,
    Necklace = 9,
    LeftRing = 10,
    RightRing = 11,
    Special = 12,
}
