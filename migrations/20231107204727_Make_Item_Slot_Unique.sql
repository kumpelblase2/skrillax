alter table character_items
    add constraint character_items_slot_uniq
        unique (character_id, slot);