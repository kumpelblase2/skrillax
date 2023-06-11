use num_enum_derive::{IntoPrimitive, TryFromPrimitive};

#[derive(IntoPrimitive, TryFromPrimitive, Copy, Clone, Eq, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum EquipmentSlot {
    HeadArmor,
    ShoulderArmor,
    WristArmor,
    ChestArmor,
    LegArmor,
    FootArmor,
    Weapon,
    SecondaryWeapon,
    Earring,
    Necklace,
    LeftRing,
    RightRing,
    Special,
}
