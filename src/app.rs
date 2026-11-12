use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::character::{
    self, Ability, Class, LevelLimit, Physical, Race, ScoreSet, Sex, HIGH_SCORE,
};

/// Startup menu options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuChoice {
    /// Walk through scores → race → class.
    StepThrough,
    /// Roll scores, race and class in one go.
    RandomCharacter,
    Quit,
}

impl MenuChoice {
    pub fn label(self) -> &'static str {
        match self {
            MenuChoice::StepThrough => "Step through",
            MenuChoice::RandomCharacter => "Random character",
            MenuChoice::Quit => "Quit",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MenuChoice::StepThrough => "roll scores, then choose race and class yourself",
            MenuChoice::RandomCharacter => "roll scores, race and class at random",
            MenuChoice::Quit => "leave without a character",
        }
    }
}

/// Startup menu entries, in display order.
pub const MENU_ITEMS: [MenuChoice; 3] = [
    MenuChoice::StepThrough,
    MenuChoice::RandomCharacter,
    MenuChoice::Quit,
];

/// What the app is currently doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Startup menu.
    Menu,
    /// Reordering the freshly rolled scores.
    OrderScores,
    /// Picking a race (chosen or random).
    ChooseRace,
    /// Picking a class from those the race and scores allow.
    ChooseClass,
    /// Final look at the character before exiting.
    Review,
    /// Everything is chosen; the app is winding down.
    Finished,
}

pub struct App {
    /// The raw 3d6/4d6 scores, in 1e order.
    pub scores: [u8; 6],
    /// Index of the highlighted score in step 1.
    pub cursor: usize,
    /// How many sets have been rolled so far (including auto re-rolls).
    pub attempts: u32,
    pub phase: Phase,
    pub should_quit: bool,
    /// Highlighted startup-menu entry.
    pub menu_cursor: usize,
    /// Highlighted race in step 2.
    pub race_cursor: usize,
    /// Highlighted class in step 3.
    pub class_cursor: usize,
    pub race: Option<Race>,
    pub class: Option<Class>,
    /// Age, height and weight, rolled when the character reaches review.
    pub physical: Option<Physical>,
    /// Explanation shown when the player picks something they cannot be.
    pub message: Option<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            scores: [0; 6],
            cursor: 0,
            attempts: 0,
            phase: Phase::Menu,
            should_quit: false,
            menu_cursor: 0,
            race_cursor: 0,
            class_cursor: 0,
            race: None,
            class: None,
            physical: None,
            message: None,
        }
    }

    /// Roll sets until at least one race/class combination is open.
    fn reroll_until_viable(&mut self) {
        for _ in 0..character::MAX_ROLL_ATTEMPTS {
            self.reroll();
            if self.any_viable_race() {
                return;
            }
        }
    }

    /// Roll scores, a race and a class, then show the result for review.
    pub fn random_character(&mut self) {
        self.reroll_until_viable();
        self.pick_random_race();
        if self.phase != Phase::ChooseClass {
            // Nothing was viable (should not happen after reroll_until_viable).
            return;
        }
        let eligible: Vec<Class> = Class::ALL
            .iter()
            .copied()
            .filter(|class| self.is_eligible(*class))
            .collect();
        if eligible.is_empty() {
            return;
        }
        let class = eligible[rand::random_range(0..eligible.len())];
        self.class = Some(class);
        self.class_cursor = Class::ALL.iter().position(|c| *c == class).unwrap_or(0);
        self.enter_review();
        self.message = None;
    }

    /// Re-roll height and weight as the other sex, keeping age.
    pub fn toggle_sex(&mut self) {
        let (Some(race), Some(class), Some(physical)) = (self.race, self.class, self.physical)
        else {
            return;
        };
        if self.phase != Phase::Review {
            return;
        }
        let mut rolled = Physical::roll(race, class, physical.sex.other());
        rolled.age = physical.age;
        rolled.category = physical.category;
        self.physical = Some(rolled);
    }

    /// Show the review screen, rolling age/height/weight for this character.
    fn enter_review(&mut self) {
        self.physical = match (self.race, self.class) {
            (Some(race), Some(class)) => Some(Physical::roll(race, class, Sex::random())),
            _ => None,
        };
        self.phase = Phase::Review;
    }

    /// Roll a brand new set (auto re-rolling until the requirement is met).
    pub fn reroll(&mut self) {
        let set = character::roll_ability_scores();
        self.scores = set.scores;
        self.attempts = set.attempts;
        self.cursor = 0;
        self.race = None;
        self.class = None;
        self.physical = None;
        self.message = None;
    }

    /// The scores exactly as rolled.
    pub fn score_set(&self) -> ScoreSet {
        ScoreSet {
            scores: self.scores,
            attempts: self.attempts,
        }
    }

    /// The scores after racial adjustments (equals [`Self::score_set`] before a
    /// race has been chosen).
    pub fn final_score_set(&self) -> ScoreSet {
        let racial = match self.race {
            Some(race) => race.apply(&self.score_set()),
            None => self.score_set(),
        };
        // Aging modifiers only exist once the character has a birthday; they
        // are applied after racial adjustments, as in 1e, and never push a
        // score below the chosen class's minimums.
        match (self.race, self.physical) {
            (Some(race), Some(physical)) => {
                let minimums: &[(Ability, u8)] = self
                    .class
                    .map(|class| class.requirements())
                    .unwrap_or(&[]);
                character::apply_aging_modifiers(
                    &racial,
                    &race.aging_modifiers(physical.age),
                    minimums,
                )
            }
            _ => racial,
        }
    }

    /// What a given race would turn the current roll into.
    pub fn score_set_for(&self, race: Race) -> ScoreSet {
        race.apply(&self.score_set())
    }

    /// Raw scores in 1e order (step 1 display).
    pub fn assignments(&self) -> impl Iterator<Item = (Ability, u8)> + '_ {
        Ability::ALL
            .iter()
            .copied()
            .zip(self.scores.iter().copied())
    }

    /// Race-adjusted scores in 1e order.
    pub fn final_assignments(&self) -> impl Iterator<Item = (Ability, u8)> {
        Ability::ALL
            .iter()
            .copied()
            .zip(self.final_score_set().scores)
    }

    pub fn high_score_count(&self) -> usize {
        self.score_set().high_score_count()
    }

    pub fn total(&self) -> u32 {
        self.scores.iter().map(|s| *s as u32).sum()
    }

    pub fn final_total(&self) -> u32 {
        self.final_score_set().total()
    }

    /// Classes the chosen race allows, ignoring ability scores.
    pub fn race_allows(&self, class: Class) -> bool {
        self.race.is_none_or(|race| race.can_be(class))
    }

    /// Racial level cap for a class, or `None` if the race cannot take it.
    pub fn level_limit(&self, class: Class) -> Option<LevelLimit> {
        self.race.and_then(|race| race.level_limit(class))
    }

    /// Class is open to the chosen race *and* the adjusted scores are enough.
    pub fn is_eligible(&self, class: Class) -> bool {
        self.race_allows(class) && class.is_eligible(&self.final_score_set())
    }

    /// How many classes are open if this race is chosen.
    pub fn viability_count(&self, race: Race) -> usize {
        let scores = self.score_set_for(race);
        Class::ALL
            .iter()
            .filter(|class| race.can_be(**class) && class.is_eligible(&scores))
            .count()
    }

    pub fn is_viable_race(&self, race: Race) -> bool {
        self.viability_count(race) > 0
    }

    pub fn any_viable_race(&self) -> bool {
        Race::ALL.iter().copied().any(|race| self.is_viable_race(race))
    }

    /// Pick a random race among those that leave at least one class open.
    pub fn pick_random_race(&mut self) {
        let viable: Vec<Race> = Race::ALL
            .iter()
            .copied()
            .filter(|race| self.is_viable_race(*race))
            .collect();
        if viable.is_empty() {
            self.message = Some(
                "No race/class combination fits these scores — press Backspace to reroll.".into(),
            );
            return;
        }
        let race = viable[rand::random_range(0..viable.len())];
        self.race = Some(race);
        self.race_cursor = Race::ALL.iter().position(|r| *r == race).unwrap_or(0);
        self.enter_class_step();
    }

    /// Move the highlighted entry in the current step.
    pub fn move_cursor(&mut self, delta: isize) {
        match self.phase {
            Phase::Menu => {
                let len = MENU_ITEMS.len() as isize;
                self.menu_cursor = (self.menu_cursor as isize + delta).rem_euclid(len) as usize;
            }
            Phase::OrderScores => {
                let len = self.scores.len() as isize;
                self.cursor = (self.cursor as isize + delta).rem_euclid(len) as usize;
            }
            Phase::ChooseRace => {
                let len = Race::ALL.len() as isize;
                self.race_cursor =
                    (self.race_cursor as isize + delta).rem_euclid(len) as usize;
                self.message = None;
            }
            Phase::ChooseClass => {
                let len = Class::ALL.len() as isize;
                self.class_cursor =
                    (self.class_cursor as isize + delta).rem_euclid(len) as usize;
                self.message = None;
            }
            Phase::Review | Phase::Finished => {}
        }
    }

    /// Swap the highlighted score with the one above/below it.
    pub fn move_score(&mut self, delta: isize) {
        if self.phase != Phase::OrderScores {
            return;
        }
        let target = self.cursor as isize + delta;
        if !(0..self.scores.len() as isize).contains(&target) {
            return;
        }
        let target = target as usize;
        self.scores.swap(self.cursor, target);
        self.cursor = target;
    }

    /// Enter the class step, highlighting the first class that is open.
    fn enter_class_step(&mut self) {
        self.phase = Phase::ChooseClass;
        self.message = None;
        if !self.is_eligible(Class::ALL[self.class_cursor]) {
            self.class_cursor = Class::ALL
                .iter()
                .position(|class| self.is_eligible(*class))
                .unwrap_or(0);
        }
    }

    /// Advance to the next step, or finish from the review screen.
    pub fn accept(&mut self) {
        match self.phase {
            Phase::Menu => {
                match MENU_ITEMS[self.menu_cursor] {
                    MenuChoice::StepThrough => {
                        self.reroll_until_viable();
                        self.phase = Phase::OrderScores;
                    }
                    MenuChoice::RandomCharacter => self.random_character(),
                    MenuChoice::Quit => self.should_quit = true,
                }
            }
            Phase::OrderScores => {
                if !self.any_viable_race() {
                    self.message = Some(
                        "No race/class combination fits these scores — press r to reroll.".into(),
                    );
                    return;
                }
                self.phase = Phase::ChooseRace;
                if !self.is_viable_race(Race::ALL[self.race_cursor]) {
                    self.race_cursor = Race::ALL
                        .iter()
                        .position(|race| self.is_viable_race(*race))
                        .unwrap_or(0);
                }
                self.message = None;
            }
            Phase::ChooseRace => {
                let race = Race::ALL[self.race_cursor];
                if !self.is_viable_race(race) {
                    self.message = Some(format!(
                        "A {} has no class open with these scores.",
                        race.label()
                    ));
                    return;
                }
                self.race = Some(race);
                self.enter_class_step();
            }
            Phase::ChooseClass => {
                let class = Class::ALL[self.class_cursor];
                if let Some(race) = self.race
                    && !race.can_be(class)
                {
                    self.message = Some(format!(
                        "{} cannot be {}.",
                        race.plural_label(),
                        class.plural_label()
                    ));
                    return;
                }
                let unmet = class.unmet_requirements(&self.final_score_set());
                if !unmet.is_empty() {
                    let reasons: Vec<String> = unmet
                        .iter()
                        .map(|(ability, required, rolled)| {
                            format!("{} {required} (have {rolled})", ability.label())
                        })
                        .collect();
                    self.message = Some(format!(
                        "Cannot be a {}: needs {}",
                        class.label(),
                        reasons.join(", ")
                    ));
                    return;
                }
                self.class = Some(class);
                self.enter_review();
                self.message = None;
            }
            Phase::Review => {
                self.phase = Phase::Finished;
            }
            Phase::Finished => {}
        }
    }

    /// Go back one step.
    pub fn back(&mut self) {
        match self.phase {
            Phase::ChooseRace => {
                self.phase = Phase::OrderScores;
                self.message = None;
            }
            Phase::ChooseClass => {
                self.phase = Phase::ChooseRace;
                self.race = None;
                self.message = None;
            }
            Phase::Review => {
                self.phase = Phase::ChooseClass;
                self.physical = None;
            }
            _ => {}
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.phase == Phase::Finished {
            return;
        }
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            // Ctrl + arrows and shift + k/j move the score itself, not the cursor.
            KeyCode::Up if key.modifiers.contains(KeyModifiers::CONTROL) => self.move_score(-1),
            KeyCode::Down if key.modifiers.contains(KeyModifiers::CONTROL) => self.move_score(1),
            KeyCode::Char('K') => self.move_score(-1),
            KeyCode::Char('J') => self.move_score(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::Char('r') if self.phase == Phase::OrderScores => self.reroll(),
            KeyCode::Char('g') if self.phase == Phase::ChooseRace => self.pick_random_race(),
            KeyCode::Char('s') if self.phase == Phase::Review => self.toggle_sex(),
            KeyCode::Backspace => self.back(),
            KeyCode::Enter => self.accept(),
            _ => {}
        }
    }

    /// Text printed after the TUI exits, if the character was finished.
    pub fn accepted_summary(&self) -> Option<String> {
        if self.phase != Phase::Finished {
            return None;
        }
        let mut out = String::new();
        if let Some(race) = self.race {
            out.push_str(&format!("Race:  {}\n", race.label()));
        }
        if let Some(class) = self.class {
            out.push_str(&format!("Class: {}\n", class.label()));
        }
        if let Some(physical) = self.physical {
            out.push_str(&format!("Sex:   {}\n", physical.sex.label()));
            out.push_str(&format!("Age:   {}\n", physical.age_label()));
            out.push_str(&format!("Height: {}\n", physical.height_label()));
            out.push_str(&format!("Weight: {} lb\n", physical.weight_lbs));
            if let Some(race) = self.race {
                let aging = physical.aging_label(race);
                if aging != "none yet" {
                    out.push_str(&format!("Aging:  {aging}\n"));
                }
            }
        }
        out.push_str("Ability scores:\n");
        for (ability, score) in self.final_assignments() {
            out.push_str(&format!(
                "  {:<13} {:>2}{}\n",
                ability.label(),
                score,
                if score >= HIGH_SCORE { " *" } else { "" }
            ));
        }
        out.push_str(&format!("  {:<13} {:>2}\n", "Total", self.final_total()));
        if let (Some(race), Some(class)) = (self.race, self.class)
            && let Some(limit) = race.level_limit(class)
        {
            out.push_str(&format!("Level limit: {}\n", limit.label()));
        }
        Some(out)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::MIN_HIGH_SCORES;
    /// App with the "step through" menu option already chosen.
    fn started() -> App {
        let mut app = App::new();
        app.accept();
        assert_eq!(app.phase, Phase::OrderScores);
        app
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn menu_app_has_no_rolls_yet() {
        let app = App::new();
        assert_eq!(app.phase, Phase::Menu);
        assert_eq!(app.attempts, 0);
        assert_eq!(app.scores, [0; 6]);
        assert_eq!(app.menu_cursor, 0);
    }

    #[test]
    fn started_app_meets_high_score_requirement() {
        let app = started();
        assert!(app.high_score_count() >= MIN_HIGH_SCORES);
        assert!(app.attempts >= 1);
    }

    #[test]
    fn menu_cursor_wraps() {
        let mut app = App::new();
        app.move_cursor(-1);
        assert_eq!(app.menu_cursor, MENU_ITEMS.len() - 1);
        app.move_cursor(1);
        assert_eq!(app.menu_cursor, 0);
    }

    #[test]
    fn menu_quit_option_quits() {
        let mut app = App::new();
        app.menu_cursor = MENU_ITEMS
            .iter()
            .position(|c| *c == MenuChoice::Quit)
            .unwrap();
        app.handle_key(key(KeyCode::Enter));
        assert!(app.should_quit);
        assert!(app.accepted_summary().is_none());
    }

    #[test]
    fn menu_random_option_builds_a_whole_character() {
        for _ in 0..25 {
            let mut app = App::new();
            app.menu_cursor = MENU_ITEMS
                .iter()
                .position(|c| *c == MenuChoice::RandomCharacter)
                .unwrap();
            app.handle_key(key(KeyCode::Enter));

            assert_eq!(app.phase, Phase::Review);
            let race = app.race.expect("race should be rolled");
            let class = app.class.expect("class should be rolled");
            assert!(race.can_be(class), "{race:?} cannot be {class:?}");
            assert!(class.is_eligible(&app.final_score_set()));
            assert!(app.race.is_some_and(|r| r.level_limit(class).is_some()));
            assert!(app.attempts >= 1);
            assert!(app.high_score_count() >= MIN_HIGH_SCORES);

            app.accept();
            assert_eq!(app.phase, Phase::Finished);
            let summary = app.accepted_summary().unwrap();
            assert!(summary.contains(race.label()), "{summary}");
            assert!(summary.contains(class.label()), "{summary}");
        }
    }

    #[test]
    fn random_character_is_always_reviewable_and_back_able() {
        let mut app = App::new();
        app.random_character();
        assert_eq!(app.phase, Phase::Review);
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::ChooseClass);
    }

    #[test]
    fn cursor_wraps_around() {
        let mut app = started();
        app.move_cursor(-1);
        assert_eq!(app.cursor, 5);
        app.move_cursor(1);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn moving_scores_swaps_without_losing_any() {
        let mut app = started();
        app.scores = [10, 11, 12, 13, 14, 15];
        app.move_score(1);
        assert_eq!(app.scores, [11, 10, 12, 13, 14, 15]);
        assert_eq!(app.cursor, 1);
        app.move_score(-1);
        assert_eq!(app.scores, [10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn ctrl_arrows_move_the_score_not_the_cursor() {
        let mut app = started();
        app.scores = [10, 11, 12, 13, 14, 15];
        app.cursor = 2;

        app.handle_key(ctrl(KeyCode::Up));
        assert_eq!(app.scores, [10, 12, 11, 13, 14, 15]);
        assert_eq!(app.cursor, 1);

        app.handle_key(ctrl(KeyCode::Down));
        assert_eq!(app.scores, [10, 11, 12, 13, 14, 15]);
        assert_eq!(app.cursor, 2);

        // Plain arrows only move the cursor.
        app.handle_key(key(KeyCode::Down));
        assert_eq!(app.scores, [10, 11, 12, 13, 14, 15]);
        assert_eq!(app.cursor, 3);
    }

    #[test]
    fn ctrl_arrows_are_ignored_outside_step_one() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        let scores = app.scores;
        app.handle_key(ctrl(KeyCode::Up));
        assert_eq!(app.scores, scores);
        assert_eq!(app.phase, Phase::ChooseClass);
    }

    #[test]
    fn moving_top_score_up_does_nothing() {
        let mut app = started();
        app.scores = [10, 11, 12, 13, 14, 15];
        app.cursor = 0;
        app.move_score(-1);
        assert_eq!(app.scores, [10, 11, 12, 13, 14, 15]);
        assert_eq!(app.cursor, 0);
    }

    #[test]
    fn q_quits_and_enter_advances() {
        let mut app = started();
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);

        let mut app = started();
        app.handle_key(key(KeyCode::Enter));
        assert_eq!(app.phase, Phase::ChooseRace);
        assert!(app.accepted_summary().is_none());
    }

    #[test]
    fn assignments_follow_ability_order() {
        let mut app = started();
        app.scores = [9, 10, 11, 12, 13, 14];
        let pairs: Vec<_> = app.assignments().collect();
        assert_eq!(pairs[0], (Ability::Strength, 9));
        assert_eq!(pairs[5], (Ability::Charisma, 14));
    }

    /// Scores good enough for every class.
    fn godly(app: &mut App) {
        app.scores = [18, 18, 18, 18, 18, 18];
    }

    fn race_index(race: Race) -> usize {
        Race::ALL.iter().position(|r| *r == race).unwrap()
    }

    fn class_index(class: Class) -> usize {
        Class::ALL.iter().position(|c| *c == class).unwrap()
    }

    /// Enter step 2 and pick a race, landing in step 3.
    fn choose_race(app: &mut App, race: Race) {
        app.accept(); // step 1 -> races
        app.race_cursor = race_index(race);
        app.accept(); // race -> classes
        assert_eq!(app.phase, Phase::ChooseClass);
        assert_eq!(app.race, Some(race));
    }

    #[test]
    fn entering_race_step_skips_to_a_viable_race() {
        let mut app = started();
        app.scores = [12, 9, 13, 10, 9, 16];
        app.accept();
        assert_eq!(app.phase, Phase::ChooseRace);
        assert!(app.is_viable_race(Race::ALL[app.race_cursor]));
    }

    #[test]
    fn race_adjustments_change_the_final_scores() {
        let mut app = started();
        app.scores = [12, 12, 12, 12, 12, 12];
        choose_race(&mut app, Race::Dwarf);
        assert_eq!(app.final_score_set().scores, [12, 12, 12, 12, 13, 11]);
        let final_pairs: Vec<_> = app.final_assignments().collect();
        assert_eq!(final_pairs[4], (Ability::Constitution, 13));
        assert_eq!(app.total(), 72, "raw total unchanged");
        assert_eq!(app.final_total(), 72, "dwarf +1/-1 cancel out");
    }

    #[test]
    fn race_restriction_blocks_a_class() {
        let mut app = started();
        app.scores = [12, 18, 12, 12, 12, 12];
        choose_race(&mut app, Race::Dwarf);
        app.class_cursor = class_index(Class::MagicUser);
        app.accept();
        assert_eq!(app.phase, Phase::ChooseClass, "must stay on the class step");
        let msg = app.message.clone().unwrap_or_default();
        assert!(msg.contains("Dwarves cannot be Magic-Users"), "{msg}");
        assert!(!app.is_eligible(Class::MagicUser));
    }

    #[test]
    fn ability_requirements_use_adjusted_scores() {
        let mut app = started();
        // Halflings get Str -1, so a rolled 9 leaves 8 — short of a Fighter's 9.
        app.scores = [9, 12, 12, 12, 12, 12];
        choose_race(&mut app, Race::Halfling);
        assert_eq!(app.final_score_set().get(Ability::Strength), 8);

        app.class_cursor = class_index(Class::Fighter);
        app.accept();
        assert_eq!(app.phase, Phase::ChooseClass);
        let msg = app.message.clone().unwrap_or_default();
        assert!(msg.contains("Strength 9 (have 8)"), "{msg}");

        // The same roll is enough for a Thief (Dex 9, +1 for halflings).
        app.class_cursor = class_index(Class::Thief);
        app.accept();
        assert_eq!(app.phase, Phase::Review);
    }

    #[test]
    fn random_race_only_picks_viable_races() {
        for _ in 0..50 {
            let mut app = started();
            godly(&mut app);
            app.accept();
            app.pick_random_race();
            assert_eq!(app.phase, Phase::ChooseClass);
            assert!(app.is_viable_race(app.race.unwrap()));
        }
    }

    #[test]
    fn random_race_refuses_when_nothing_is_viable() {
        let mut app = started();
        app.scores = [3, 3, 3, 3, 3, 3];
        app.phase = Phase::ChooseRace;
        app.pick_random_race();
        assert_eq!(app.phase, Phase::ChooseRace);
        assert!(app.race.is_none());
        assert!(app.message.is_some());
    }

    #[test]
    fn random_race_advances_to_classes() {
        let mut app = started();
        godly(&mut app);
        app.accept();
        app.pick_random_race();
        assert_eq!(app.phase, Phase::ChooseClass);
        assert!(app.race.is_some());
        assert!(app.class_cursor < Class::ALL.len());
        assert!(app.is_eligible(Class::ALL[app.class_cursor]));
    }

    #[test]
    fn race_with_no_open_class_is_rejected() {
        let mut app = started();
        // Humans can be clerics here; a halfling cannot be anything.
        app.scores = [3, 12, 12, 3, 12, 12];
        app.accept();
        assert_eq!(app.phase, Phase::ChooseRace);
        app.race_cursor = race_index(Race::Halfling);
        app.accept();
        assert_eq!(app.phase, Phase::ChooseRace);
        let msg = app.message.clone().unwrap_or_default();
        assert!(msg.contains("Halfling"), "{msg}");
        assert!(msg.contains("no class open"), "{msg}");
    }

    #[test]
    fn review_rolls_age_height_and_weight() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        assert!(app.physical.is_none(), "no body data before review");

        app.class_cursor = class_index(Class::Fighter);
        app.accept();
        assert_eq!(app.phase, Phase::Review);
        let physical = app.physical.expect("review should roll age/height/weight");
        assert!(physical.age >= 15);
        assert!(physical.height_inches > 0);
        assert!(physical.weight_lbs > 0);
        assert_eq!(
            physical.category,
            crate::character::AgeCategory::from_age(Race::Human, physical.age)
        );

        // Going back discards it; coming forward again rolls a new one.
        app.handle_key(key(KeyCode::Backspace));
        assert!(app.physical.is_none());
        app.accept();
        assert!(app.physical.is_some());
    }

    #[test]
    fn toggle_sex_keeps_age_but_rerolls_the_body() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        app.class_cursor = class_index(Class::Fighter);
        app.accept();
        let before = app.physical.unwrap();

        // Sample a few toggles so height/weight changes are all but certain.
        let mut changed = false;
        for _ in 0..8 {
            let sex = app.physical.unwrap().sex;
            app.handle_key(key(KeyCode::Char('s')));
            let after = app.physical.unwrap();
            assert_eq!(after.sex, sex.other());
            assert_eq!(after.age, before.age, "age must not change");
            assert_eq!(after.category, before.category);
            changed |= after.height_inches != before.height_inches
                || after.weight_lbs != before.weight_lbs;
        }
        assert!(changed, "height/weight should be re-rolled");
    }

    #[test]
    fn sex_toggle_only_works_on_the_review_screen() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        assert!(app.physical.is_none());
        app.toggle_sex();
        assert!(app.physical.is_none());
    }

    #[test]
    fn physical_data_is_in_the_summary() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Elf);
        app.class_cursor = class_index(Class::Thief);
        app.accept();
        app.accept();
        assert_eq!(app.phase, Phase::Finished);

        let summary = app.accepted_summary().unwrap();
        assert!(summary.contains("Sex:"), "{summary}");
        assert!(summary.contains("Age:"), "{summary}");
        assert!(summary.contains("Height:"), "{summary}");
        assert!(summary.contains("Weight:"), "{summary}");
    }

    #[test]
    fn random_character_also_gets_a_body() {
        let mut app = App::new();
        app.random_character();
        assert_eq!(app.phase, Phase::Review);
        assert!(app.physical.is_some());
    }

    #[test]
    fn reroll_clears_physical_data() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        app.class_cursor = class_index(Class::Fighter);
        app.accept();
        assert!(app.physical.is_some());
        app.handle_key(key(KeyCode::Backspace));
        app.handle_key(key(KeyCode::Backspace));
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::OrderScores);
        app.handle_key(key(KeyCode::Char('r')));
        assert!(app.physical.is_none());
    }

    #[test]
    fn level_limit_is_exposed_for_the_chosen_race() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Dwarf);
        assert_eq!(app.level_limit(Class::Fighter), Some(LevelLimit::Max(9)));
        assert_eq!(app.level_limit(Class::MagicUser), None);

        app.class_cursor = class_index(Class::Fighter);
        app.accept();
        app.accept();
        let summary = app.accepted_summary().unwrap();
        assert!(summary.contains("Level limit: max 9"), "{summary}");
    }

    #[test]
    fn choosing_a_class_and_finishing() {
        let mut app = started();
        good_but_not_paladin(&mut app);
        choose_race(&mut app, Race::Human);
        app.class_cursor = class_index(Class::Paladin);
        app.accept();
        assert_eq!(app.phase, Phase::ChooseClass, "paladin must be rejected");
        let msg = app.message.clone().unwrap_or_default();
        assert!(msg.contains("Charisma 17 (have 16)"), "{msg}");

        app.class_cursor = class_index(Class::Thief);
        app.accept();
        assert_eq!(app.phase, Phase::Review);
        assert_eq!(app.class, Some(Class::Thief));

        app.accept();
        assert_eq!(app.phase, Phase::Finished);
        let summary = app.accepted_summary().unwrap();
        assert!(summary.contains("Race:  Human"), "{summary}");
        assert!(summary.contains("Class: Thief"), "{summary}");
    }

    /// Str 12, Int 9, Wis 13, Dex 10, Con 9, Cha 16: everything but Paladin.
    fn good_but_not_paladin(app: &mut App) {
        app.scores = [12, 9, 13, 10, 9, 16];
    }

    #[test]
    fn back_from_classes_clears_the_race() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Elf);
        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::ChooseRace);
        assert_eq!(app.race, None);

        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::OrderScores);
    }

    #[test]
    fn review_can_go_back_to_class_choice() {
        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        app.class_cursor = class_index(Class::Thief);
        app.accept();
        assert_eq!(app.phase, Phase::Review);

        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::ChooseClass);

        app.handle_key(key(KeyCode::Backspace));
        assert_eq!(app.phase, Phase::ChooseRace);
    }

    #[test]
    fn reroll_is_ignored_outside_step_one() {
        let mut app = started();
        app.accept();
        let before = app.scores;
        app.handle_key(key(KeyCode::Char('r')));
        assert_eq!(app.scores, before);
    }

    #[test]
    fn g_rolls_a_random_race_in_step_two() {
        let mut app = started();
        godly(&mut app);
        app.accept();
        app.handle_key(key(KeyCode::Char('g')));
        assert_eq!(app.phase, Phase::ChooseClass);
        assert!(app.race.is_some());
    }

    #[test]
    fn quit_still_works_in_every_step() {
        let mut app = started();
        app.accept();
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);

        let mut app = started();
        choose_race(&mut app, Race::Human);
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);

        let mut app = started();
        godly(&mut app);
        choose_race(&mut app, Race::Human);
        app.class = Some(Class::Thief);
        app.phase = Phase::Review;
        app.handle_key(key(KeyCode::Char('q')));
        assert!(app.should_quit);
        assert!(app.accepted_summary().is_none());
    }
}
