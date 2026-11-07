use std::collections::HashMap;
pub enum Race{
    Dwarf, Elf, Gnome, HalfElf, Halfling, HalfOrc,  Human,
}

pub enum Class{
    Cleric, Druid, Fighter, Paladin, Ranger, MagicUser, Illusionist, Thief, Assassin,
}

pub enum Ability {
    Strength, Intelligence, Wisdom, Dexterity, Constitution, Charisma
}

pub struct Character {
    name: String,
    race: Race,
    class: Class,
    abilities: HashMap<Ability, u8>,
    //01-100
    exceptional_strength: u8,
    encumberunce: u32,
}

pub struct Attack {
    name: String,
    damage_min: u8,
    damage_max: u16
}

pub struct Monster {
    name: String,
    type_name: String,
    hit_dice: u8,
    hit_points: u16,
    armor_class: i8,
    attacks: Vec<Attack>,
}

pub struct Paramaters {
    race: Option<Race>,
    class: Vec<Class>,

}

pub fn create_character(param: Paramaters) {


}

pub select_race() -> Race {
    
}

pub fn roll_ability() -> u8 {
    0
}

pub fn roll_dice(start: u8, end:u8) -> u8 {

}
