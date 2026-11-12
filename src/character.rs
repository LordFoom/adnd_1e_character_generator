use std::collections::HashMap;

use crate::character::Race::Human;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Race{
    Dwarf,
    /// Wherever a table splits elves into sub-races, the high elf row is used.
    Elf,
    Gnome,
    HalfElf,
    Halfling,
    HalfOrc,
    Human,
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
                (Monk, Unlimited),
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
        apply_modifiers(scores, self.adjustments())
    }

    /// Starting age from the DMG table, in years.
    ///
    /// Demihuman ages are given per class for cleric / fighter / magic-user /
    /// thief. Classes that do not appear in those columns borrow the closest
    /// one: druid → cleric, paladin and ranger → fighter, illusionist →
    /// magic-user, assassin → thief.
    pub fn starting_age(self, class: Class) -> DiceRoll {
        match self {
            Race::Human => match class {
                Class::Cleric => DiceRoll::new(18, 1, 4),
                Class::Druid => DiceRoll::new(18, 1, 4),
                Class::Fighter => DiceRoll::new(15, 1, 4),
                Class::Paladin => DiceRoll::new(17, 1, 4),
                Class::Ranger => DiceRoll::new(20, 1, 4),
                Class::MagicUser => DiceRoll::new(24, 2, 8),
                Class::Illusionist => DiceRoll::new(30, 1, 6),
                Class::Thief => DiceRoll::new(18, 1, 4),
                Class::Assassin => DiceRoll::new(20, 1, 4),
                Class::Monk => DiceRoll::new(21, 1, 4),
            },
            _ => {
                let column = match class {
                    Class::Cleric | Class::Druid => 0,
                    Class::Fighter | Class::Paladin | Class::Ranger => 1,
                    Class::MagicUser | Class::Illusionist => 2,
                    Class::Thief | Class::Assassin | Class::Monk => 3,
                };
                self.demihuman_starting_age(column)
            }
        }
    }

    /// One of the four demihuman age columns; a dash in the table means the
    /// race cannot take that class, so the fighter column is the fallback.
    fn demihuman_starting_age(self, column: usize) -> DiceRoll {
        use DiceRoll as D;
        let row: [Option<DiceRoll>; 4] = match self {
            Race::Dwarf => [
                Some(D::new(250, 2, 20)),
                Some(D::new(40, 5, 4)),
                None,
                Some(D::new(75, 3, 6)),
            ],
            Race::Elf => [
                Some(D::new(500, 10, 10)),
                Some(D::new(130, 5, 6)),
                Some(D::new(150, 5, 6)),
                Some(D::new(100, 5, 6)),
            ],
            Race::Gnome => [
                Some(D::new(300, 3, 12)),
                Some(D::new(60, 5, 4)),
                Some(D::new(100, 2, 12)),
                Some(D::new(80, 5, 4)),
            ],
            Race::HalfElf => [
                Some(D::new(40, 2, 4)),
                Some(D::new(22, 3, 4)),
                Some(D::new(30, 2, 8)),
                Some(D::new(22, 3, 8)),
            ],
            Race::Halfling => [None, Some(D::new(20, 3, 4)), None, Some(D::new(40, 2, 4))],
            Race::HalfOrc => [
                Some(D::new(20, 1, 4)),
                Some(D::new(13, 1, 4)),
                None,
                Some(D::new(20, 2, 4)),
            ],
            Race::Human => [None, None, None, None],
        };
        row[column].unwrap_or(DiceRoll::new(20, 1, 4))
    }

    /// Inclusive age ranges for young adult, mature, middle aged, old and
    /// venerable characters, from the DMG age category table.
    ///
    /// The elven row below is the high elf row: when a table distinguishes elf
    /// sub-races (or lists only sub-races), high elf is always the one used.
    pub fn age_ranges(self) -> [(u16, u16); 5] {
        match self {
            Race::Dwarf => [(35, 50), (51, 150), (151, 250), (251, 350), (351, 450)],
            Race::Elf => [(100, 175), (176, 550), (551, 875), (876, 1200), (1201, 1600)],
            Race::Gnome => [(50, 90), (91, 300), (301, 450), (451, 600), (601, 750)],
            Race::HalfElf => [(24, 40), (41, 100), (101, 175), (176, 250), (251, 325)],
            Race::Halfling => [(22, 33), (34, 68), (69, 101), (102, 144), (145, 199)],
            Race::HalfOrc => [(12, 15), (16, 30), (31, 45), (46, 60), (61, 80)],
            Race::Human => [(14, 20), (21, 40), (41, 60), (61, 90), (91, 120)],
        }
    }

    /// Average height in inches with its minus/plus dice, by sex.
    pub fn height_profile(self, sex: Sex) -> BodySize {
        use DiceRoll as D;
        match (self, sex) {
            (Race::Dwarf, Sex::Male) => BodySize::new(48, D::new(0, 1, 4), D::new(0, 1, 6)),
            (Race::Dwarf, Sex::Female) => BodySize::new(46, D::new(0, 1, 4), D::new(0, 1, 4)),
            (Race::Elf, Sex::Male) => BodySize::new(60, D::new(0, 1, 4), D::new(0, 1, 6)),
            (Race::Elf, Sex::Female) => BodySize::new(54, D::new(0, 1, 4), D::new(0, 1, 6)),
            (Race::Gnome, Sex::Male) => BodySize::new(42, D::new(0, 1, 3), D::new(0, 1, 3)),
            (Race::Gnome, Sex::Female) => BodySize::new(39, D::new(0, 1, 3), D::new(0, 1, 3)),
            (Race::HalfElf, Sex::Male) => BodySize::new(66, D::new(0, 1, 6), D::new(0, 1, 6)),
            (Race::HalfElf, Sex::Female) => BodySize::new(62, D::new(0, 1, 6), D::new(0, 1, 6)),
            (Race::Halfling, Sex::Male) => BodySize::new(36, D::new(0, 1, 3), D::new(0, 1, 6)),
            (Race::Halfling, Sex::Female) => BodySize::new(33, D::new(0, 1, 3), D::new(0, 1, 3)),
            (Race::HalfOrc, Sex::Male) => BodySize::new(66, D::new(0, 1, 4), D::new(0, 1, 4)),
            (Race::HalfOrc, Sex::Female) => BodySize::new(62, D::new(0, 1, 3), D::new(0, 1, 3)),
            (Race::Human, Sex::Male) => BodySize::new(72, D::new(0, 1, 12), D::new(0, 1, 12)),
            (Race::Human, Sex::Female) => BodySize::new(66, D::new(0, 1, 6), D::new(0, 1, 8)),
        }
    }

    /// Average weight in pounds with its minus/plus dice, by sex.
    pub fn weight_profile(self, sex: Sex) -> BodySize {
        use DiceRoll as D;
        match (self, sex) {
            (Race::Dwarf, Sex::Male) => BodySize::new(150, D::new(0, 2, 8), D::new(0, 2, 12)),
            (Race::Dwarf, Sex::Female) => BodySize::new(120, D::new(0, 2, 8), D::new(0, 2, 10)),
            (Race::Elf, Sex::Male) => BodySize::new(100, D::new(0, 1, 10), D::new(0, 1, 20)),
            (Race::Elf, Sex::Female) => BodySize::new(80, D::new(0, 1, 10), D::new(0, 2, 6)),
            (Race::Gnome, Sex::Male) => BodySize::new(80, D::new(0, 2, 4), D::new(0, 2, 6)),
            (Race::Gnome, Sex::Female) => BodySize::new(75, D::new(0, 1, 8), D::new(0, 1, 8)),
            (Race::HalfElf, Sex::Male) => BodySize::new(130, D::new(0, 1, 20), D::new(0, 1, 20)),
            (Race::HalfElf, Sex::Female) => BodySize::new(100, D::new(0, 1, 12), D::new(0, 2, 8)),
            (Race::Halfling, Sex::Male) => BodySize::new(60, D::new(0, 2, 4), D::new(0, 2, 6)),
            (Race::Halfling, Sex::Female) => BodySize::new(50, D::new(0, 2, 4), D::new(0, 2, 4)),
            (Race::HalfOrc, Sex::Male) => BodySize::new(150, D::new(0, 2, 8), D::new(0, 4, 10)),
            (Race::HalfOrc, Sex::Female) => BodySize::new(120, D::new(0, 3, 6), D::new(0, 4, 8)),
            (Race::Human, Sex::Male) => BodySize::new(175, D::new(0, 3, 12), D::new(0, 5, 12)),
            (Race::Human, Sex::Female) => BodySize::new(130, D::new(0, 3, 10), D::new(0, 4, 12)),
        }
    }

    /// d100 bands for under / average / over height.
    pub fn height_bands(self) -> [(u8, u8); 3] {
        match self {
            Race::Dwarf => [(1, 15), (16, 80), (81, 100)],
            Race::Elf => [(1, 10), (11, 80), (81, 100)],
            Race::Gnome => [(1, 20), (21, 85), (86, 100)],
            Race::HalfElf => [(1, 35), (36, 90), (91, 100)],
            Race::Halfling => [(1, 10), (11, 90), (91, 100)],
            Race::HalfOrc => [(1, 45), (46, 75), (76, 100)],
            Race::Human => [(1, 20), (21, 80), (81, 100)],
        }
    }

    /// d100 bands for under / average / over weight.
    pub fn weight_bands(self) -> [(u8, u8); 3] {
        match self {
            Race::Dwarf => [(1, 20), (21, 65), (66, 100)],
            Race::Elf => [(1, 15), (16, 90), (91, 100)],
            Race::Gnome => [(1, 20), (21, 75), (76, 100)],
            Race::HalfElf => [(1, 20), (21, 85), (86, 100)],
            Race::Halfling => [(1, 10), (11, 50), (51, 100)],
            Race::HalfOrc => [(1, 30), (31, 55), (56, 100)],
            Race::Human => [(1, 25), (26, 75), (76, 100)],
        }
    }

    /// Every age category the character has passed through adds its modifiers.
    pub fn aging_modifiers(self, age: u16) -> Vec<(Ability, i8)> {
        let mut totals: Vec<(Ability, i8)> = Vec::new();
        for (category, (start, _)) in AgeCategory::ALL.iter().zip(self.age_ranges()) {
            if age < start {
                continue;
            }
            for (ability, delta) in category.modifiers() {
                match totals.iter_mut().find(|(a, _)| a == ability) {
                    Some((_, total)) => *total += delta,
                    None => totals.push((*ability, *delta)),
                }
            }
        }
        totals.retain(|(_, total)| *total != 0);
        totals
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
    Cleric, Druid, Fighter, Paladin, Ranger, MagicUser, Illusionist, Thief, Assassin, Monk,
}

impl Class {
    /// All selectable classes, in the order shown to the player.
    pub const ALL: [Class; 10] = [
        Class::Cleric,
        Class::Druid,
        Class::Fighter,
        Class::Paladin,
        Class::Ranger,
        Class::MagicUser,
        Class::Illusionist,
        Class::Thief,
        Class::Assassin,
        Class::Monk,
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
            Class::Monk => "Monk",
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
            Class::Monk => "Monks",
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
            // Monks are human-only, so the race table below has no entry for
            // them under any demihuman race.
            Class::Monk => &[
                (Strength, 15),
                (Wisdom, 15),
                (Dexterity, 15),
                (Constitution, 11),
            ],
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

/// A dice expression: `base + count` d `sides`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiceRoll {
    pub base: u16,
    pub count: u8,
    pub sides: u8,
}

impl DiceRoll {
    pub const fn new(base: u16, count: u8, sides: u8) -> Self {
        Self { base, count, sides }
    }

    pub fn roll(&self) -> u16 {
        let total: u16 = (0..self.count)
            .map(|_| roll_dice(1, self.sides) as u16)
            .sum();
        self.base + total
    }

    /// Text form, e.g. `15+1d4`.
    pub fn label(&self) -> String {
        match (self.count, self.sides) {
            (0, _) => self.base.to_string(),
            (count, sides) => format!("{}+{count}d{sides}", self.base),
        }
    }
}

/// Age brackets that carry ability score modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgeCategory {
    YoungAdult,
    Mature,
    MiddleAge,
    Old,
    Venerable,
}

impl AgeCategory {
    /// In order of increasing age.
    pub const ALL: [AgeCategory; 5] = [
        AgeCategory::YoungAdult,
        AgeCategory::Mature,
        AgeCategory::MiddleAge,
        AgeCategory::Old,
        AgeCategory::Venerable,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AgeCategory::YoungAdult => "young adult",
            AgeCategory::Mature => "mature",
            AgeCategory::MiddleAge => "middle age",
            AgeCategory::Old => "old age",
            AgeCategory::Venerable => "venerable",
        }
    }

    /// Modifiers for entering this category; they accumulate with age.
    pub fn modifiers(self) -> &'static [(Ability, i8)] {
        use Ability::*;
        match self {
            AgeCategory::YoungAdult => &[(Wisdom, -1), (Constitution, 1)],
            AgeCategory::Mature => &[(Strength, 1), (Wisdom, 1)],
            AgeCategory::MiddleAge => &[
                (Strength, -1),
                (Constitution, -1),
                (Intelligence, 1),
                (Wisdom, 1),
            ],
            AgeCategory::Old => &[
                (Strength, -2),
                (Dexterity, -2),
                (Constitution, -1),
                (Wisdom, 1),
            ],
            AgeCategory::Venerable => &[
                (Strength, -1),
                (Dexterity, -1),
                (Constitution, -1),
                (Intelligence, 1),
                (Wisdom, 1),
            ],
        }
    }

    /// The category a character of this race and age falls into.
    pub fn from_age(race: Race, age: u16) -> Self {
        let mut category = AgeCategory::YoungAdult;
        for (index, (start, _)) in race.age_ranges().iter().enumerate() {
            if age >= *start {
                category = AgeCategory::ALL[index];
            }
        }
        category
    }
}

/// Sex, needed for the height and weight tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sex {
    Male,
    Female,
}

impl Sex {
    pub const ALL: [Sex; 2] = [Sex::Male, Sex::Female];

    pub fn label(self) -> &'static str {
        match self {
            Sex::Male => "male",
            Sex::Female => "female",
        }
    }

    pub fn other(self) -> Self {
        match self {
            Sex::Male => Sex::Female,
            Sex::Female => Sex::Male,
        }
    }

    pub fn random() -> Self {
        if rand::random::<bool>() {
            Sex::Male
        } else {
            Sex::Female
        }
    }
}

/// An average height or weight plus its minus/plus dice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BodySize {
    pub average: u16,
    pub minus: DiceRoll,
    pub plus: DiceRoll,
}

impl BodySize {
    pub const fn new(average: u16, minus: DiceRoll, plus: DiceRoll) -> Self {
        Self {
            average,
            minus,
            plus,
        }
    }
}

/// Races shorter than five feet use the smaller average-height variation.
pub const SMALL_FRAME_INCHES: u16 = 60;
/// Weight at or below this uses the smaller average-weight variation.
pub const LIGHT_FRAME_LBS: u16 = 100;
/// Wisdom is the only score aging may push above 18.
pub const MAX_WISDOM_SCORE: u8 = 25;

/// Roll a d100 (1..=100).
fn roll_percentile() -> u8 {
    roll_dice(1, 100)
}

/// Which column (0 = under, 1 = average, 2 = over) a d100 falls in.
fn band_of(roll: u8, bands: [(u8, u8); 3]) -> usize {
    bands
        .iter()
        .position(|(low, high)| (*low..=*high).contains(&roll))
        .unwrap_or(1)
}

/// Height in inches: under/average/over bands, where the average band rolls
/// again for a 1-4% (1-3% if under five feet) height variation.
pub fn roll_height(race: Race, sex: Sex) -> u16 {
    let profile = race.height_profile(sex);
    match band_of(roll_percentile(), race.height_bands()) {
        0 => profile.average.saturating_sub(profile.minus.roll()).max(1),
        2 => profile.average + profile.plus.roll(),
        _ => {
            let variation = if profile.average < SMALL_FRAME_INCHES {
                roll_dice(1, 3)
            } else {
                roll_dice(1, 4)
            } as u16;
            match roll_percentile() {
                1..=30 => profile.average - profile.average * variation / 100,
                71..=100 => profile.average + profile.average * variation / 100,
                _ => profile.average,
            }
        }
    }
}

/// Weight in pounds: under/average/over bands, where the average band rolls
/// again for a 1-8 lb (1-4 lb if 100 lb or less) variation.
pub fn roll_weight(race: Race, sex: Sex) -> u16 {
    let profile = race.weight_profile(sex);
    match band_of(roll_percentile(), race.weight_bands()) {
        0 => profile.average.saturating_sub(profile.minus.roll()).max(1),
        2 => profile.average + profile.plus.roll(),
        _ => {
            let variation = if profile.average <= LIGHT_FRAME_LBS {
                roll_dice(1, 4)
            } else {
                roll_dice(1, 8)
            } as u16;
            match roll_percentile() {
                1..=30 => profile.average.saturating_sub(variation).max(1),
                71..=100 => profile.average + variation,
                _ => profile.average,
            }
        }
    }
}

/// Height, weight, sex and age rolled for a finished race/class combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Physical {
    pub sex: Sex,
    pub age: u16,
    pub height_inches: u16,
    pub weight_lbs: u16,
    pub category: AgeCategory,
}

impl Physical {
    /// Roll age (race and class starting-age dice), height and weight.
    pub fn roll(race: Race, class: Class, sex: Sex) -> Self {
        let age = race.starting_age(class).roll();
        Self {
            sex,
            age,
            height_inches: roll_height(race, sex),
            weight_lbs: roll_weight(race, sex),
            category: AgeCategory::from_age(race, age),
        }
    }

    /// e.g. `5'11"`.
    pub fn height_label(&self) -> String {
        format!("{}'{}\"", self.height_inches / 12, self.height_inches % 12)
    }

    /// e.g. `23 (young adult)`.
    pub fn age_label(&self) -> String {
        format!("{} ({})", self.age, self.category.label())
    }

    /// Text list of the age modifiers currently in force.
    pub fn aging_label(&self, race: Race) -> String {
        let modifiers = race.aging_modifiers(self.age);
        if modifiers.is_empty() {
            return "none".to_string();
        }
        modifiers
            .iter()
            .map(|(ability, delta)| format!("{} {:+}", ability.abbrev(), delta))
            .collect::<Vec<_>>()
            .join("  ")
    }
}

/// Apply a list of ability modifiers, clamping to [`MIN_SCORE`]..=[`MAX_SCORE`].
pub fn apply_modifiers(scores: &ScoreSet, modifiers: &[(Ability, i8)]) -> ScoreSet {
    let mut adjusted = ScoreSet {
        scores: scores.scores,
        attempts: scores.attempts,
    };
    for (ability, delta) in modifiers {
        let index = Ability::ALL
            .iter()
            .position(|a| a == ability)
            .expect("every ability is in Ability::ALL");
        let value = adjusted.scores[index] as i16 + *delta as i16;
        adjusted.scores[index] = value.clamp(MIN_SCORE as i16, MAX_SCORE as i16) as u8;
    }
    adjusted
}

/// Apply aging modifiers. Scores may not fall below the class minimums and
/// only Wisdom may rise above 18, as the DMG notes require.
pub fn apply_aging_modifiers(
    scores: &ScoreSet,
    modifiers: &[(Ability, i8)],
    minimums: &[(Ability, u8)],
) -> ScoreSet {
    let mut adjusted = ScoreSet {
        scores: scores.scores,
        attempts: scores.attempts,
    };
    for (ability, delta) in modifiers {
        let index = Ability::ALL
            .iter()
            .position(|a| a == ability)
            .expect("every ability is in Ability::ALL");
        let floor = minimums
            .iter()
            .find(|(minimum_ability, _)| minimum_ability == ability)
            .map(|(_, minimum)| *minimum)
            .unwrap_or(MIN_SCORE)
            .max(MIN_SCORE);
        let cap = if *ability == Ability::Wisdom {
            MAX_WISDOM_SCORE
        } else {
            MAX_SCORE
        };
        let value = adjusted.scores[index] as i16 + *delta as i16;
        adjusted.scores[index] = value.clamp(floor as i16, cap as i16) as u8;
    }
    adjusted
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
    fn monk_is_human_only() {
        assert!(Race::Human.can_be(Class::Monk));
        assert_eq!(Race::Human.level_limit(Class::Monk), Some(LevelLimit::Unlimited));
        for race in Race::ALL {
            if race != Race::Human {
                assert!(!race.can_be(Class::Monk), "{race:?} should not be a monk");
            }
        }
    }

    #[test]
    fn monk_needs_str_wis_dex_fifteen_and_con_eleven() {
        // Str 15, Int ?, Wis 15, Dex 15, Con 11, Cha ?
        let qualifies = set(15, 9, 15, 15, 11, 9);
        assert!(Class::Monk.is_eligible(&qualifies));

        for (scores, missing) in [
            (set(14, 9, 15, 15, 11, 9), Ability::Strength),
            (set(15, 9, 14, 15, 11, 9), Ability::Wisdom),
            (set(15, 9, 15, 14, 11, 9), Ability::Dexterity),
            (set(15, 9, 15, 15, 10, 9), Ability::Constitution),
        ] {
            assert!(!Class::Monk.is_eligible(&scores), "{missing:?} should be short");
            assert_eq!(Class::Monk.unmet_requirements(&scores).len(), 1);
        }
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
    fn dice_roll_is_base_plus_nd_sides() {
        for _ in 0..2000 {
            let r = DiceRoll::new(10, 3, 6).roll();
            assert!((13..=28).contains(&r), "rolled {r}");
        }
        assert_eq!(DiceRoll::new(7, 0, 6).roll(), 7);
        assert_eq!(DiceRoll::new(15, 1, 4).label(), "15+1d4");
        assert_eq!(DiceRoll::new(7, 0, 6).label(), "7");
    }

    #[test]
    fn starting_ages_follow_the_dmg_table() {
        assert_eq!(Race::Dwarf.starting_age(Class::Fighter).label(), "40+5d4");
        assert_eq!(Race::Dwarf.starting_age(Class::Cleric).label(), "250+2d20");
        assert_eq!(Race::Dwarf.starting_age(Class::Thief).label(), "75+3d6");
        assert_eq!(Race::Elf.starting_age(Class::MagicUser).label(), "150+5d6");
        assert_eq!(Race::Elf.starting_age(Class::Cleric).label(), "500+10d10");
        assert_eq!(Race::Gnome.starting_age(Class::Illusionist).label(), "100+2d12");
        assert_eq!(Race::HalfElf.starting_age(Class::Ranger).label(), "22+3d4");
        assert_eq!(Race::HalfElf.starting_age(Class::Druid).label(), "40+2d4");
        assert_eq!(Race::Halfling.starting_age(Class::Thief).label(), "40+2d4");
        assert_eq!(Race::HalfOrc.starting_age(Class::Cleric).label(), "20+1d4");

        // Humans are listed per class.
        assert_eq!(Race::Human.starting_age(Class::Fighter).label(), "15+1d4");
        assert_eq!(Race::Human.starting_age(Class::MagicUser).label(), "24+2d8");
        assert_eq!(Race::Human.starting_age(Class::Illusionist).label(), "30+1d6");
        assert_eq!(Race::Human.starting_age(Class::Monk).label(), "21+1d4");
    }

    #[test]
    fn every_race_rolls_a_sensible_body() {
        for race in Race::ALL {
            for sex in Sex::ALL {
                for class in Class::ALL {
                    let physical = Physical::roll(race, class, sex);
                    assert!(physical.age >= race.starting_age(class).base, "{race:?}");
                    assert!(physical.height_inches > 0, "{race:?}");
                    assert!(physical.weight_lbs > 0, "{race:?}");
                    assert_eq!(
                        physical.category,
                        AgeCategory::from_age(race, physical.age)
                    );
                    assert_eq!(physical.sex, sex);
                }
            }
        }
    }

    #[test]
    fn height_and_weight_stay_near_the_table_average() {
        for race in Race::ALL {
            for sex in Sex::ALL {
                let height = race.height_profile(sex);
                let weight = race.weight_profile(sex);

                let (min_h, max_h) = (height.average - height.minus.sides as u16, height.average + height.plus.sides as u16);
                let (min_w, max_w) = (weight.average.saturating_sub(weight.minus.sides as u16 * weight.minus.count as u16), weight.average + weight.plus.sides as u16 * weight.plus.count as u16);

                for _ in 0..200 {
                    let h = roll_height(race, sex);
                    let w = roll_weight(race, sex);
                    assert!((min_h..=max_h).contains(&h), "{race:?} {sex:?} height {h}");
                    assert!((min_w..=max_w).contains(&w), "{race:?} {sex:?} weight {w}");
                }
            }
        }
    }

    #[test]
    fn percentile_bands_cover_every_roll() {
        for race in Race::ALL {
            for bands in [race.height_bands(), race.weight_bands()] {
                assert_eq!(bands[0].0, 1, "{race:?}");
                assert_eq!(bands[2].1, 100, "{race:?}");
                assert_eq!(bands[0].1 + 1, bands[1].0, "{race:?}");
                assert_eq!(bands[1].1 + 1, bands[2].0, "{race:?}");
            }
        }
    }

    #[test]
    fn females_are_on_average_smaller_than_males() {
        for race in Race::ALL {
            assert!(
                race.height_profile(Sex::Female).average < race.height_profile(Sex::Male).average,
                "{race:?}"
            );
            assert!(
                race.weight_profile(Sex::Female).average < race.weight_profile(Sex::Male).average,
                "{race:?}"
            );
        }
    }

    #[test]
    fn human_age_categories_follow_the_table() {
        let cases = [
            (13, AgeCategory::YoungAdult),
            (14, AgeCategory::YoungAdult),
            (20, AgeCategory::YoungAdult),
            (21, AgeCategory::Mature),
            (40, AgeCategory::Mature),
            (41, AgeCategory::MiddleAge),
            (60, AgeCategory::MiddleAge),
            (61, AgeCategory::Old),
            (90, AgeCategory::Old),
            (91, AgeCategory::Venerable),
            (400, AgeCategory::Venerable),
        ];
        for (age, expected) in cases {
            assert_eq!(AgeCategory::from_age(Race::Human, age), expected, "age {age}");
        }
    }

    #[test]
    fn age_categories_are_race_scaled() {
        // Dwarf: 35-50, 51-150, 151-250, 251-350, 351+
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 40), AgeCategory::YoungAdult);
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 60), AgeCategory::Mature);
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 200), AgeCategory::MiddleAge);
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 300), AgeCategory::Old);
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 400), AgeCategory::Venerable);
        // Half-orc: 12-15, 16-30, 31-45, 46-60, 61+
        assert_eq!(AgeCategory::from_age(Race::HalfOrc, 14), AgeCategory::YoungAdult);
        assert_eq!(AgeCategory::from_age(Race::HalfOrc, 70), AgeCategory::Venerable);
        // A 91-year-old human is venerable; a dwarf of the same age is mature.
        assert_eq!(AgeCategory::from_age(Race::Human, 91), AgeCategory::Venerable);
        assert_eq!(AgeCategory::from_age(Race::Dwarf, 91), AgeCategory::Mature);
    }

    #[test]
    fn aging_modifiers_accumulate_with_age() {
        fn mods(age: u16) -> Vec<(Ability, i8)> {
            Race::Human.aging_modifiers(age)
        }

        // Young adult: Wis -1, Con +1.
        let young = mods(16);
        assert_eq!(young.len(), 2, "{young:?}");
        assert!(young.contains(&(Ability::Wisdom, -1)));
        assert!(young.contains(&(Ability::Constitution, 1)));

        // Mature adds Str +1 and Wis +1, cancelling the young adult Wis penalty.
        let mature = mods(25);
        assert!(mature.contains(&(Ability::Strength, 1)), "{mature:?}");
        assert!(mature.contains(&(Ability::Constitution, 1)), "{mature:?}");
        assert!(!mature.iter().any(|(a, _)| *a == Ability::Wisdom), "{mature:?}");

        // Middle age.
        let middle = mods(45);
        assert!(middle.contains(&(Ability::Intelligence, 1)), "{middle:?}");
        assert!(middle.contains(&(Ability::Wisdom, 1)), "{middle:?}");

        // Old age stacks on everything before it.
        let old = mods(70);
        assert!(old.contains(&(Ability::Strength, -2)), "{old:?}");
        assert!(old.contains(&(Ability::Dexterity, -2)), "{old:?}");
        assert!(old.contains(&(Ability::Constitution, -1)), "{old:?}");
        assert!(old.contains(&(Ability::Wisdom, 2)), "{old:?}");

        let venerable = mods(95);
        assert!(venerable.contains(&(Ability::Strength, -3)), "{venerable:?}");
        assert!(venerable.contains(&(Ability::Dexterity, -3)), "{venerable:?}");
        assert!(venerable.contains(&(Ability::Intelligence, 2)), "{venerable:?}");
        assert!(venerable.contains(&(Ability::Wisdom, 3)), "{venerable:?}");
    }

    #[test]
    fn dwarfs_and_elves_age_on_their_own_scale() {
        assert!(Race::Elf.aging_modifiers(150).len() <= 2, "high elves start young");
        assert!(!Race::Elf.aging_modifiers(600).is_empty());
        // A dwarf cleric starts around 250-290, i.e. middle aged or old.
        let dwarf_age = Race::Dwarf.starting_age(Class::Cleric).base;
        assert!(matches!(
            AgeCategory::from_age(Race::Dwarf, dwarf_age),
            AgeCategory::MiddleAge | AgeCategory::Old
        ));
        assert!(!Race::Dwarf.aging_modifiers(dwarf_age).is_empty());
    }

    #[test]
    fn aging_respects_class_minimums_and_the_wisdom_cap() {
        let scores = set(3, 18, 18, 3, 3, 18);
        let cleric = Class::Cleric.requirements();
        let aged = apply_aging_modifiers(&scores, &Race::Human.aging_modifiers(95), cleric);
        assert!(aged.get(Ability::Wisdom) >= 9, "wisdom floor violated");

        // Only wisdom may exceed 18 through aging.
        let mut scores = set(18, 18, 18, 18, 18, 18);
        scores.scores[2] = 18;
        let aged = apply_aging_modifiers(&scores, &Race::Human.aging_modifiers(95), &[]);
        assert!(aged.get(Ability::Wisdom) > 18);
        assert!(aged.get(Ability::Intelligence) <= MAX_SCORE);
        assert!(aged.get(Ability::Strength) <= MAX_SCORE);

        let aged = apply_modifiers(&scores, &Race::Human.aging_modifiers(95));
        assert!(aged.scores.iter().all(|s| (MIN_SCORE..=MAX_SCORE).contains(s)));
    }

    #[test]
    fn physical_labels_are_readable() {
        let physical = Physical {
            sex: Sex::Female,
            age: 23,
            height_inches: 71,
            weight_lbs: 168,
            category: AgeCategory::Mature,
        };
        assert_eq!(physical.height_label(), "5'11\"");
        assert_eq!(physical.age_label(), "23 (mature)");
        assert_eq!(Sex::Male.label(), "male");
        assert_eq!(Sex::Male.other(), Sex::Female);
        assert!(physical.aging_label(Race::Human).contains("Str +1"));
    }

    #[test]
    fn every_class_has_requirements_and_a_name() {
        for class in Class::ALL {
            assert!(!class.requirements().is_empty(), "{class:?}");
            assert!(!class.label().is_empty(), "{class:?}");
        }
    }
}
