use std::collections::HashMap;

use crate::character::Race::Human;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race{
    Dwarf, Elf, Gnome, HalfElf, Halfling, HalfOrc,  Human,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class{
    Cleric, Druid, Fighter, Paladin, Ranger, MagicUser, Illusionist, Thief, Assassin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ability {
    Strength, Intelligence, Wisdom, Dexterity, Constitution, Charisma
}

impl Ability {
    /// Abilities in the order AD&amp;D 1e rolls them.
    pub const ALL: [Ability; 6] = [
        Ability::Strength,
        Ability::Intelligence,
        Ability::Wisdom,
        Ability::Dexterity,
        Ability::Constitution,
        Ability::Charisma,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Ability::Strength => "Strength",
            Ability::Intelligence => "Intelligence",
            Ability::Wisdom => "Wisdom",
            Ability::Dexterity => "Dexterity",
            Ability::Constitution => "Constitution",
            Ability::Charisma => "Charisma",
        }
    }
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
    //optional give starting xp
    //first roll the stats
        //Allow rearaanging or re-rolling
    //then select race
    //then select single or dual classed


}

pub fn select_race() -> Race {
    let roll = rand::random_range(1..=10);
    if roll <= 8 {
        Human
    } else {
        let demi_human_roll = rand::random_range(1..=6);
        match demi_human_roll {
            1 => Race::Dwarf,
            2 => Race::Elf,
            3 => Race::Gnome,
            4 => Race::HalfElf,
            5 => Race::Halfling,
            6 => Race::HalfOrc,
            num => panic!("Should not have been able to select race with id: {num}" );
        }
    }
}

/// Roll a single die with an inclusive range, e.g. `roll_dice(1, 6)` is 1d6.
///
/// # Panics
/// Panics if `start > end`.
pub fn roll_dice(start: u8, end: u8) -> u8 {
    assert!(start <= end, "roll_dice: start ({start}) must be <= end ({end})");
    rand::random_range(start..=end)
}

/// Roll one ability score using the default method: 4d6, drop the lowest die
/// (range 3..=18, mean ~12.24).
pub fn roll_ability() -> u8 {
    let mut dice = [
        roll_dice(1, 6),
        roll_dice(1, 6),
        roll_dice(1, 6),
        roll_dice(1, 6),
    ];
    dice.sort_unstable();
    dice[1..].iter().sum()
}

/// Roll one ability score using the classic 3d6 method from the 1e PHB.
pub fn roll_ability_classic() -> u8 {
    roll_dice(1, 6) + roll_dice(1, 6) + roll_dice(1, 6)
}

/// Minimum number of rolled scores that must be 16 or better.
pub const MIN_HIGH_SCORES: usize = 2;
/// Score at or above which a roll counts as "high".
pub const HIGH_SCORE: u8 = 16;
/// Safety valve so a truly unlucky streak can never hang the program.
pub const MAX_ROLL_ATTEMPTS: u32 = 1_000;

/// A freshly rolled set of six ability scores, in 1e order
/// (Strength, Intelligence, Wisdom, Dexterity, Constitution, Charisma).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreSet {
    pub scores: [u8; 6],
    /// How many sets were rolled before one satisfied the high-score rule.
    pub attempts: u32,
}

impl ScoreSet {
    /// Number of scores that are 16 or better.
    pub fn high_score_count(&self) -> usize {
        self.scores
            .iter()
            .filter(|s| **s >= HIGH_SCORE)
            .count()
    }

    pub fn meets_requirement(&self) -> bool {
        self.high_score_count() >= MIN_HIGH_SCORES
    }

    pub fn total(&self) -> u32 {
        self.scores.iter().map(|s| *s as u32).sum()
    }
}

/// Roll six ability scores, re-rolling the whole set until at least
/// [`MIN_HIGH_SCORES`] of them are 16 or better.
pub fn roll_ability_scores() -> ScoreSet {
    let mut attempts = 0;
    loop {
        attempts += 1;
        let mut scores = [0u8; 6];
        for score in &mut scores {
            *score = roll_ability();
        }
        let set = ScoreSet { scores, attempts };
        if set.meets_requirement() || attempts >= MAX_ROLL_ATTEMPTS {
            return set;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_dice_stays_in_range() {
        for _ in 0..10_000 {
            let r = roll_dice(1, 6);
            assert!((1..=6).contains(&r), "rolled {r}");
        }
    }

    #[test]
    fn roll_ability_is_4d6_drop_lowest() {
        for _ in 0..10_000 {
            let a = roll_ability();
            assert!((3..=18).contains(&a), "rolled {a}");
        }
    }

    #[test]
    fn roll_ability_classic_is_3d6() {
        for _ in 0..10_000 {
            let a = roll_ability_classic();
            assert!((3..=18).contains(&a), "rolled {a}");
        }
    }

    #[test]
    fn drop_lowest_beats_classic_on_average() {
        let n = 20_000;
        let classic: u32 = (0..n).map(|_| roll_ability_classic() as u32).sum();
        let drop: u32 = (0..n).map(|_| roll_ability() as u32).sum();
        assert!(drop > classic, "drop-lowest mean was not higher");
    }

    #[test]
    fn roll_dice_single_value_range() {
        assert_eq!(roll_dice(4, 4), 4);
    }

    #[test]
    fn roll_ability_actually_varies() {
        let mut seen = [false; 19];
        for _ in 0..10_000 {
            seen[roll_ability() as usize] = true;
        }
        assert!(seen.iter().filter(|s| **s).count() > 10);
    }

    #[test]
    fn score_set_always_meets_requirement() {
        for _ in 0..500 {
            let set = roll_ability_scores();
            assert!(set.meets_requirement(), "{set:?}");
            assert!(set.attempts >= 1);
        }
    }

    #[test]
    fn high_score_count_counts_sixteens() {
        let set = ScoreSet { scores: [16, 18, 15, 9, 16, 3], attempts: 1 };
        assert_eq!(set.high_score_count(), 3);
        assert!(set.meets_requirement());
        assert_eq!(set.total(), 77);
    }
}
