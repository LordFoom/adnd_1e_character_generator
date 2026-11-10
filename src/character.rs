use std::collections::HashMap;

use crate::character::Race::Human;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race{
    Dwarf, Elf, Gnome, HalfElf, Halfling, HalfOrc,  Human,
}

impl Race {
    /// All playable races, in the order shown to the player.
    pub const ALL: [Race; 7] = [
        Race::Human,
        Race::Dwarf,
        Race::Elf,
        Race::Gnome,
        Race::HalfElf,
        Race::Halfling,
        Race::HalfOrc,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Race::Dwarf => "Dwarf",
            Race::Elf => "Elf",
            Race::Gnome => "Gnome",
            Race::HalfElf => "Half-Elf",
            Race::Halfling => "Halfling",
            Race::HalfOrc => "Half-Orc",
            Race::Human => "Human",
        }
    }

    /// Plural form, used in messages like "Dwarves cannot be Magic-Users".
    pub fn plural_label(self) -> &'static str {
        match self {
            Race::Dwarf => "Dwarves",
            Race::Elf => "Elves",
            Race::Gnome => "Gnomes",
            Race::HalfElf => "Half-Elves",
            Race::Halfling => "Halflings",
            Race::HalfOrc => "Half-Orcs",
            Race::Human => "Humans",
        }
    }

    /// Ability score adjustments, per the 1e PHB racial table.
    pub fn adjustments(self) -> &'static [(Ability, i8)] {
        use Ability::*;
        match self {
            Race::Human => &[],
            Race::Dwarf => &[(Constitution, 1), (Charisma, -1)],
            Race::Elf => &[(Dexterity, 1), (Constitution, -1)],
            Race::Gnome => &[],
            Race::HalfElf => &[],
            Race::Halfling => &[(Dexterity, 1), (Strength, -1)],
            Race::HalfOrc => &[(Strength, 1), (Constitution, 1), (Charisma, -2)],
        }
    }

    /// Racial level limits from the 1e PHB table: the maximum level this race
    /// may reach in each class, and which classes it may take at all. A class
    /// missing from this list is not open to the race.
    pub fn class_limits(self) -> &'static [(Class, LevelLimit)] {
        use Class::*;
        use LevelLimit::{Max, Unlimited};
        match self {
            Race::Human => &[
                (Cleric, Unlimited),
                (Druid, Unlimited),
                (Fighter, Unlimited),
                (Paladin, Unlimited),
                (Ranger, Unlimited),
                (MagicUser, Unlimited),
                (Illusionist, Unlimited),
                (Thief, Unlimited),
                (Assassin, Unlimited),
            ],
            Race::Dwarf => &[
                (Cleric, Max(8)),
                (Fighter, Max(9)),
                (Thief, Max(12)),
                (Assassin, Max(9)),
            ],
            Race::Elf => &[
                (Cleric, Max(7)),
                (Fighter, Max(7)),
                (MagicUser, Max(11)),
                (Thief, Max(12)),
                (Assassin, Max(9)),
            ],
            Race::Gnome => &[
                (Cleric, Max(7)),
                (Fighter, Max(6)),
                (Illusionist, Max(7)),
                (Thief, Max(12)),
                (Assassin, Max(9)),
            ],
            Race::HalfElf => &[
                (Cleric, Max(5)),
                (Druid, Max(6)),
                (Fighter, Max(8)),
                (Ranger, Max(8)),
                (MagicUser, Max(8)),
                (Thief, Max(12)),
                (Assassin, Max(9)),
            ],
            Race::Halfling => &[(Fighter, Max(6)), (Thief, Max(12))],
            Race::HalfOrc => &[
                (Cleric, Max(4)),
                (Fighter, Max(10)),
                (Thief, Max(12)),
                (Assassin, Max(8)),
            ],
        }
    }

    /// Maximum level in one class, or `None` if the race cannot take it.
    pub fn level_limit(self, class: Class) -> Option<LevelLimit> {
        self.class_limits()
            .iter()
            .find(|(candidate, _)| *candidate == class)
            .map(|(_, limit)| *limit)
    }

    /// Classes this race may take, in the order of [`Race::class_limits`].
    pub fn allowed_classes(self) -> Vec<Class> {
        self.class_limits().iter().map(|(class, _)| *class).collect()
    }

    pub fn can_be(self, class: Class) -> bool {
        self.level_limit(class).is_some()
    }

    /// Apply this race's adjustments to a fresh set of scores.
    pub fn apply(self, scores: &ScoreSet) -> ScoreSet {
        let mut adjusted = ScoreSet {
            scores: scores.scores,
            attempts: scores.attempts,
        };
        for (ability, delta) in self.adjustments() {
            let index = Ability::ALL
                .iter()
                .position(|a| a == ability)
                .expect("every ability is in Ability::ALL");
            let value = adjusted.scores[index] as i16 + *delta as i16;
            adjusted.scores[index] = value.clamp(MIN_SCORE as i16, MAX_SCORE as i16) as u8;
        }
        adjusted
    }

    /// Short text form of the adjustments, e.g. `Con +1  Cha -1`.
    pub fn adjustment_summary(self) -> String {
        if self.adjustments().is_empty() {
            return "no adjustments".to_string();
        }
        self.adjustments()
            .iter()
            .map(|(ability, delta)| format!("{} {:+}", ability.abbrev(), delta))
            .collect::<Vec<_>>()
            .join("  ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Class{
    Cleric, Druid, Fighter, Paladin, Ranger, MagicUser, Illusionist, Thief, Assassin,
}

impl Class {
    /// All selectable classes, in the order shown to the player.
    pub const ALL: [Class; 9] = [
        Class::Cleric,
        Class::Druid,
        Class::Fighter,
        Class::Paladin,
        Class::Ranger,
        Class::MagicUser,
        Class::Illusionist,
        Class::Thief,
        Class::Assassin,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Class::Cleric => "Cleric",
            Class::Druid => "Druid",
            Class::Fighter => "Fighter",
            Class::Paladin => "Paladin",
            Class::Ranger => "Ranger",
            Class::MagicUser => "Magic-User",
            Class::Illusionist => "Illusionist",
            Class::Thief => "Thief",
            Class::Assassin => "Assassin",
        }
    }

    /// Plural form, used in messages like "Dwarves cannot be Magic-Users".
    pub fn plural_label(self) -> &'static str {
        match self {
            Class::Cleric => "Clerics",
            Class::Druid => "Druids",
            Class::Fighter => "Fighters",
            Class::Paladin => "Paladins",
            Class::Ranger => "Rangers",
            Class::MagicUser => "Magic-Users",
            Class::Illusionist => "Illusionists",
            Class::Thief => "Thieves",
            Class::Assassin => "Assassins",
        }
    }

    /// Minimum ability scores from the 1e PHB, Table II.
    pub fn requirements(self) -> &'static [(Ability, u8)] {
        use Ability::*;
        match self {
            Class::Cleric => &[(Wisdom, 9)],
            Class::Druid => &[(Wisdom, 12), (Charisma, 15)],
            Class::Fighter => &[(Strength, 9), (Constitution, 7)],
            Class::Paladin => &[
                (Strength, 12),
                (Intelligence, 9),
                (Wisdom, 13),
                (Constitution, 9),
                (Charisma, 17),
            ],
            Class::Ranger => &[
                (Strength, 13),
                (Intelligence, 13),
                (Wisdom, 14),
                (Constitution, 14),
            ],
            Class::MagicUser => &[(Intelligence, 9), (Dexterity, 6)],
            Class::Illusionist => &[(Intelligence, 15), (Dexterity, 16)],
            Class::Thief => &[(Dexterity, 9)],
            Class::Assassin => &[(Strength, 12), (Intelligence, 11), (Dexterity, 12)],
        }
    }

    /// Requirements this character fails, as `(ability, required, rolled)`.
    pub fn unmet_requirements(self, scores: &ScoreSet) -> Vec<(Ability, u8, u8)> {
        self.requirements()
            .iter()
            .filter(|(ability, min)| scores.get(*ability) < *min)
            .map(|(ability, min)| (*ability, *min, scores.get(*ability)))
            .collect()
    }

    pub fn is_eligible(self, scores: &ScoreSet) -> bool {
        self.unmet_requirements(scores).is_empty()
    }
}

/// How far a demihuman can advance in a class (1e racial level limits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LevelLimit {
    /// Humans have no racial cap.
    Unlimited,
    /// Demihumans stop at this level.
    Max(u8),
}

impl LevelLimit {
    /// Text form used in the TUI, e.g. `max 8` or `unlimited`.
    pub fn label(self) -> String {
        match self {
            LevelLimit::Unlimited => "unlimited".to_string(),
            LevelLimit::Max(level) => format!("max {level}"),
        }
    }

    /// Compact form, e.g. `U` or `8`.
    pub fn short(self) -> String {
        match self {
            LevelLimit::Unlimited => "U".to_string(),
            LevelLimit::Max(level) => level.to_string(),
        }
    }
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

    /// Three-letter abbreviation used in class requirement lists.
    pub fn abbrev(self) -> &'static str {
        match self {
            Ability::Strength => "Str",
            Ability::Intelligence => "Int",
            Ability::Wisdom => "Wis",
            Ability::Dexterity => "Dex",
            Ability::Constitution => "Con",
            Ability::Charisma => "Cha",
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
            num => panic!("Should not have been able to select race with id: {num}"),
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
/// Lowest ability score an adjustment can leave behind.
pub const MIN_SCORE: u8 = 3;
/// Highest normal ability score (exceptional strength is a separate roll).
pub const MAX_SCORE: u8 = 18;
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

    /// Score for one ability, using the 1e roll order of [`Ability::ALL`].
    pub fn get(&self, ability: Ability) -> u8 {
        let index = Ability::ALL
            .iter()
            .position(|a| *a == ability)
            .expect("every ability is in Ability::ALL");
        self.scores[index]
    }

    /// Abilities this set qualifies for.
    pub fn eligible_classes(&self) -> Vec<Class> {
        Class::ALL
            .iter()
            .copied()
            .filter(|class| class.is_eligible(self))
            .collect()
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

    fn set(str_: u8, int: u8, wis: u8, dex: u8, con: u8, cha: u8) -> ScoreSet {
        ScoreSet { scores: [str_, int, wis, dex, con, cha], attempts: 1 }
    }

    #[test]
    fn get_uses_1e_order() {
        let s = set(1, 2, 3, 4, 5, 6);
        assert_eq!(s.get(Ability::Strength), 1);
        assert_eq!(s.get(Ability::Intelligence), 2);
        assert_eq!(s.get(Ability::Wisdom), 3);
        assert_eq!(s.get(Ability::Dexterity), 4);
        assert_eq!(s.get(Ability::Constitution), 5);
        assert_eq!(s.get(Ability::Charisma), 6);
    }

    #[test]
    fn cleric_needs_wisdom_nine() {
        assert!(Class::Cleric.is_eligible(&set(9, 9, 9, 9, 9, 9)));
        assert!(!Class::Cleric.is_eligible(&set(18, 18, 8, 18, 18, 18)));
    }

    #[test]
    fn paladin_is_hard_to_qualify_for() {
        // Needs Str 12, Int 9, Wis 13, Con 9, Cha 17.
        let almost = set(12, 9, 13, 10, 9, 16);
        assert!(!Class::Paladin.is_eligible(&almost));
        assert_eq!(
            Class::Paladin.unmet_requirements(&almost),
            vec![(Ability::Charisma, 17, 16)]
        );

        let qualifies = set(12, 9, 13, 10, 9, 17);
        assert!(Class::Paladin.is_eligible(&qualifies));
    }

    #[test]
    fn illusionist_needs_int_and_dex() {
        assert!(!Class::Illusionist.is_eligible(&set(18, 15, 18, 15, 18, 18)));
        assert_eq!(
            Class::Illusionist.unmet_requirements(&set(18, 15, 18, 15, 18, 18)),
            vec![(Ability::Dexterity, 16, 15)]
        );
        assert!(Class::Illusionist.is_eligible(&set(18, 15, 18, 16, 18, 18)));
    }

    #[test]
    fn eligible_classes_filters_by_requirements() {
        let all_threes = set(3, 3, 3, 3, 3, 3);
        assert!(all_threes.eligible_classes().is_empty());

        let all_eights = set(8, 8, 8, 8, 8, 8);
        assert!(all_eights.eligible_classes().is_empty());

        let all_eighteens = set(18, 18, 18, 18, 18, 18);
        assert_eq!(all_eighteens.eligible_classes().len(), Class::ALL.len());
    }

    #[test]
    fn human_has_no_adjustments() {
        let s = set(12, 12, 12, 12, 12, 12);
        assert_eq!(Race::Human.apply(&s).scores, s.scores);
        assert_eq!(Race::Human.adjustment_summary(), "no adjustments");
    }

    #[test]
    fn dwarf_raises_con_and_lowers_cha() {
        let s = set(12, 12, 12, 12, 12, 12);
        assert_eq!(Race::Dwarf.apply(&s).scores, [12, 12, 12, 12, 13, 11]);
        assert_eq!(Race::Dwarf.adjustment_summary(), "Con +1  Cha -1");
    }

    #[test]
    fn halfling_raises_dex_and_lowers_str() {
        let s = set(12, 12, 12, 12, 12, 12);
        assert_eq!(Race::Halfling.apply(&s).scores, [11, 12, 12, 13, 12, 12]);
    }

    #[test]
    fn half_orc_adjusts_three_scores() {
        let s = set(12, 12, 12, 12, 12, 12);
        assert_eq!(Race::HalfOrc.apply(&s).scores, [13, 12, 12, 12, 13, 10]);
    }

    #[test]
    fn adjustments_are_clamped_to_the_usual_range() {
        let low = set(3, 3, 3, 3, 3, 3);
        assert_eq!(Race::HalfOrc.apply(&low).scores[5], 3, "cha would go to 1");

        let high = set(18, 18, 18, 18, 18, 18);
        assert_eq!(Race::HalfOrc.apply(&high).scores[0], 18, "str capped at 18");
        assert_eq!(Race::HalfOrc.apply(&high).scores[4], 18, "con capped at 18");
    }

    #[test]
    fn humans_can_take_every_class() {
        for class in Class::ALL {
            assert!(Race::Human.can_be(class), "{class:?}");
        }
    }

    #[test]
    fn humans_have_no_level_caps() {
        for class in Class::ALL {
            assert_eq!(
                Race::Human.level_limit(class),
                Some(LevelLimit::Unlimited),
                "{class:?}"
            );
        }
    }

    #[test]
    fn racial_level_limits_match_the_1e_table() {
        assert_eq!(Race::Dwarf.level_limit(Class::Cleric), Some(LevelLimit::Max(8)));
        assert_eq!(Race::Dwarf.level_limit(Class::Fighter), Some(LevelLimit::Max(9)));
        assert_eq!(Race::Elf.level_limit(Class::MagicUser), Some(LevelLimit::Max(11)));
        assert_eq!(Race::Elf.level_limit(Class::Fighter), Some(LevelLimit::Max(7)));
        assert_eq!(Race::Gnome.level_limit(Class::Illusionist), Some(LevelLimit::Max(7)));
        assert_eq!(Race::HalfElf.level_limit(Class::Ranger), Some(LevelLimit::Max(8)));
        assert_eq!(Race::Halfling.level_limit(Class::Thief), Some(LevelLimit::Max(12)));
        assert_eq!(Race::HalfOrc.level_limit(Class::Fighter), Some(LevelLimit::Max(10)));
        assert_eq!(Race::HalfOrc.level_limit(Class::Assassin), Some(LevelLimit::Max(8)));
    }

    #[test]
    fn classes_are_limited_by_the_level_limit_table() {
        assert_eq!(Race::Halfling.allowed_classes(), vec![Class::Fighter, Class::Thief]);
        assert_eq!(Race::Halfling.level_limit(Class::Paladin), None);
        assert_eq!(Race::Elf.level_limit(Class::Paladin), None);
        assert_eq!(Race::Dwarf.level_limit(Class::MagicUser), None);
        assert_eq!(Race::HalfElf.level_limit(Class::Paladin), None);
    }

    #[test]
    fn level_limit_labels() {
        assert_eq!(LevelLimit::Max(8).label(), "max 8");
        assert_eq!(LevelLimit::Unlimited.label(), "unlimited");
        assert_eq!(LevelLimit::Max(12).short(), "12");
        assert_eq!(LevelLimit::Unlimited.short(), "U");
    }

    #[test]
    fn race_class_restrictions() {
        assert!(!Race::Elf.can_be(Class::Paladin));
        assert!(!Race::Dwarf.can_be(Class::MagicUser));
        assert!(!Race::Halfling.can_be(Class::Cleric));
        assert!(!Race::HalfElf.can_be(Class::Paladin));
        assert!(Race::HalfElf.can_be(Class::Ranger));
        assert!(Race::Gnome.can_be(Class::Illusionist));
        assert!(Race::Dwarf.can_be(Class::Assassin));
    }

    #[test]
    fn every_race_has_some_class_and_a_name() {
        for race in Race::ALL {
            assert!(!race.allowed_classes().is_empty(), "{race:?}");
            assert!(!race.allowed_classes().len() > Class::ALL.len());
            assert!(!race.label().is_empty(), "{race:?}");
        }
    }

    #[test]
    fn adjustments_never_panic_on_extremes() {
        for race in Race::ALL {
            let low = race.apply(&set(3, 3, 3, 3, 3, 3));
            let high = race.apply(&set(18, 18, 18, 18, 18, 18));
            for score in low.scores.iter().chain(high.scores.iter()) {
                assert!((MIN_SCORE..=MAX_SCORE).contains(score), "{score} out of range");
            }
        }
    }

    #[test]
    fn every_class_has_requirements_and_a_name() {
        for class in Class::ALL {
            assert!(!class.requirements().is_empty(), "{class:?}");
            assert!(!class.label().is_empty(), "{class:?}");
        }
    }
}
