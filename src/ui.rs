use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};

use crate::app::{App, MenuChoice, Phase, MENU_ITEMS};
use crate::character::{Ability, Class, LevelLimit, Race, HIGH_SCORE, MIN_HIGH_SCORES};

const ACCENT: Color = Color::Yellow;
const LABEL: Color = Color::Cyan;
const MUTED: Color = Color::DarkGray;

pub fn draw(frame: &mut Frame, app: &App) {
    let [header_area, body_area, help_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(10),
        Constraint::Length(5),
    ])
    .areas(frame.area());

    draw_header(frame, header_area, app);
    match app.phase {
        Phase::Menu => draw_menu(frame, body_area, app),
        Phase::OrderScores => draw_score_ordering(frame, body_area, app),
        Phase::ChooseRace => draw_race_selection(frame, body_area, app),
        Phase::ChooseClass => draw_class_selection(frame, body_area, app),
        Phase::Review | Phase::Finished => draw_review(frame, body_area, app),
    }
    draw_help(frame, help_area, app);
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App) {
    let step = match app.phase {
        Phase::Menu => "Start",
        Phase::OrderScores => "Step 1 · Ability Scores",
        Phase::ChooseRace => "Step 2 · Race",
        Phase::ChooseClass => "Step 3 · Class",
        Phase::Review | Phase::Finished => "Step 4 · Review",
    };

    let line = Line::from(vec![
        Span::styled(
            " AD&D 1e ",
            Style::new().fg(Color::Black).bg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Character Generation", Style::new().fg(Color::Gray)),
    ]);
    let block = Block::bordered().title(top_left(step));

    // No rolls exist yet on the menu, and narrow terminals cannot fit the box.
    if app.phase == Phase::Menu || area.width < 72 {
        frame.render_widget(Paragraph::new(line).block(block), area);
        return;
    }

    let [left, right] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Length(30)]).areas(area);
    frame.render_widget(Paragraph::new(line).block(block), left);

    let stats = Line::from(vec![
        Span::styled(" rolls ", Style::new().fg(MUTED)),
        Span::styled(
            app.attempts.to_string(),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  total ", Style::new().fg(MUTED)),
        Span::styled(
            format!("{}/108 ", app.total()),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])
    .right_aligned();
    frame.render_widget(Paragraph::new(stats).block(Block::bordered()), right);
}

fn top_left(step: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!(" {step} "),
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
    ))
}

fn draw_menu(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .map(|choice| {
            let name_style = match choice {
                MenuChoice::Quit => Style::new().fg(Color::Gray),
                _ => Style::new().fg(LABEL).add_modifier(Modifier::BOLD),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<20}", choice.label()), name_style),
                Span::styled(choice.description(), Style::new().fg(MUTED)),
            ]))
        })
        .collect();

    let mut block = Block::bordered().title(" What would you like to do? ");
    if area.width >= 60 {
        block = block.title_top(
            Line::from(Span::styled(
                " numbers, dice and a bit of luck ",
                Style::new().fg(MUTED),
            ))
            .right_aligned(),
        );
    }

    let list = List::new(items)
        .block(block)
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::new()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.menu_cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_score_ordering(frame: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .assignments()
        .map(|(ability, score)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<14}", ability.label()), Style::new().fg(LABEL)),
                Span::styled(format!("{score:>2}"), score_style(score)),
                Span::styled(star(score), Style::new().fg(score_style(score).fg.unwrap_or(MUTED))),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Assign each score to an ability ")
                .title_top(high_score_counter(app, area.width)),
        )
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::new()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.cursor));
    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_race_selection(frame: &mut Frame, area: Rect, app: &App) {
    let (preview_area, list_area) = if area.width >= 76 {
        let [l, r] = Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(area);
        (Some(l), r)
    } else {
        (None, area)
    };

    if let Some(preview_area) = preview_area {
        draw_race_preview(frame, preview_area, app);
    }

    let items: Vec<ListItem> = Race::ALL
        .iter()
        .map(|race| {
            let viable = app.is_viable_race(*race);
            let name_style = if viable {
                Style::new().fg(LABEL).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(MUTED)
            };

            let adjustments = if race.adjustments().is_empty() {
                Span::styled("no adjustments", Style::new().fg(MUTED))
            } else {
                Span::styled(
                    race.adjustment_summary(),
                    Style::new().fg(Color::Gray),
                )
            };

            let open = app.viability_count(*race);
            let open_style = if open > 0 {
                Style::new().fg(Color::Green)
            } else {
                Style::new().fg(Color::Red)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<11}", race.label()), name_style),
                adjustments,
                Span::styled(format!("   {open} classes open"), open_style),
            ]))
        })
        .collect();

    let viable = Race::ALL
        .iter()
        .filter(|race| app.is_viable_race(**race))
        .count();
    let counter_text = if area.width >= 60 {
        format!(" {viable} of {} races viable ", Race::ALL.len())
    } else {
        format!(" {viable}/{} ", Race::ALL.len())
    };

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Race · adjustments and open classes ")
                .title_top(
                    Line::from(Span::styled(
                        counter_text,
                        Style::new().fg(if viable > 0 { Color::Green } else { Color::Red }),
                    ))
                    .right_aligned(),
                )
                .title_bottom(race_list_footer(app)),
        )
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::new()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.race_cursor));
    frame.render_stateful_widget(list, list_area, &mut state);
}

/// Bottom title of the race list: the rejection message when there is one,
/// otherwise the random-pick hint.
fn race_list_footer(app: &App) -> Line<'static> {
    match &app.message {
        Some(message) => Line::from(Span::styled(
            format!(" {message} "),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        None => Line::from(Span::styled(
            " g picks a random viable race ",
            Style::new().fg(MUTED),
        )),
    }
}

fn draw_race_preview(frame: &mut Frame, area: Rect, app: &App) {
    let race = Race::ALL[app.race_cursor];
    let adjusted = app.score_set_for(race);

    let lines: Vec<Line> = Ability::ALL
        .iter()
        .map(|ability| {
            let base = app.score_set().get(*ability);
            let value = adjusted.get(*ability);
            let delta = value as i8 - base as i8;
            let delta_span = match delta {
                0 => Span::raw("    "),
                d if d > 0 => Span::styled(format!(" {d:+}"), Style::new().fg(Color::Green)),
                d => Span::styled(format!(" {d:+}"), Style::new().fg(Color::Red)),
            };
            Line::from(vec![
                Span::styled(format!(" {:<14}", ability.abbrev()), Style::new().fg(MUTED)),
                Span::styled(format!("{value:>2}"), score_style(value)),
                delta_span,
            ])
        })
        .collect();

    frame.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .title(format!(" Scores as {} ", race.label()))
                .title_bottom(Line::from(Span::styled(
                    format!(" {} ", race.adjustment_summary()),
                    Style::new().fg(MUTED),
                ))),
        ),
        area,
    );
}

fn draw_class_selection(frame: &mut Frame, area: Rect, app: &App) {
    let (scores_area, list_area) = if area.width >= 76 {
        let [l, r] = Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(area);
        (Some(l), r)
    } else {
        (None, area)
    };

    if let Some(scores_area) = scores_area {
        draw_score_summary(frame, scores_area, app);
    }

    let scores = app.final_score_set();
    let items: Vec<ListItem> = Class::ALL
        .iter()
        .map(|class| {
            let allowed = app.race_allows(*class);
            let eligible = app.is_eligible(*class);
            let name_style = if eligible {
                Style::new().fg(LABEL).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(MUTED)
            };

            let mut spans = vec![Span::styled(format!("{:<12}", class.label()), name_style)];
            for (ability, min) in class.requirements() {
                let have = scores.get(*ability);
                let style = if !allowed {
                    Style::new().fg(MUTED)
                } else if have >= *min {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Red)
                };
                spans.push(Span::styled(
                    format!("{} {:>2}  ", ability.abbrev(), min),
                    style,
                ));
            }

            spans.push(if !allowed {
                Span::styled("✘ race", Style::new().fg(Color::Magenta))
            } else if eligible {
                Span::styled("✔", Style::new().fg(Color::Green))
            } else {
                Span::styled("✘", Style::new().fg(Color::Red))
            });

            // Racial level cap, e.g. "max 9". Humans are unlimited, which is
            // the default, so they get no extra noise here.
            if let Some(LevelLimit::Max(level)) = app.level_limit(*class) {
                spans.push(Span::styled(
                    format!("  max {level}"),
                    Style::new().fg(MUTED),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    let eligible = Class::ALL.iter().filter(|c| app.is_eligible(**c)).count();
    let counter_text = if area.width >= 60 {
        format!(" {eligible} of {} classes qualify ", Class::ALL.len())
    } else {
        format!(" {eligible}/{} ", Class::ALL.len())
    };
    let counter = Line::from(Span::styled(
        counter_text,
        Style::new().fg(if eligible > 0 { Color::Green } else { Color::Red }),
    ))
    .right_aligned();

    let list = List::new(items)
        .block(
            Block::bordered()
                .title(" Class · ability minimums ")
                .title_top(counter)
                .title_bottom(list_footer(app)),
        )
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::new()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default().with_selected(Some(app.class_cursor));
    frame.render_stateful_widget(list, list_area, &mut state);
}

fn draw_review(frame: &mut Frame, area: Rect, app: &App) {
    if area.width < 76 {
        let mut lines = vec![
            character_line(app),
            race_line(app),
            level_limit_line(app),
            physical_line(app, "  Sex          ", |p| p.sex.label().to_string()),
            physical_line(app, "  Age          ", |p| p.age_label()),
            physical_line(app, "  Height       ", |p| p.height_label()),
            physical_line(app, "  Weight       ", |p| format!("{} lb", p.weight_lbs)),
            aging_line(app),
        ];
        lines.push(Line::from(""));
        lines.extend(ability_lines(app));
        lines.push(Line::from(""));
        lines.push(total_line(app));
        frame.render_widget(
            Paragraph::new(lines).block(Block::bordered().title(" Character ")),
            area,
        );
        return;
    }

    let [left, right] =
        Layout::horizontal([Constraint::Length(38), Constraint::Fill(1)]).areas(area);

    let mut sheet = vec![
        character_line(app),
        race_line(app),
        level_limit_line(app),
        physical_line(app, "  Sex          ", |p| p.sex.label().to_string()),
        physical_line(app, "  Age          ", |p| p.age_label()),
        physical_line(app, "  Height       ", |p| p.height_label()),
        physical_line(app, "  Weight       ", |p| format!("{} lb", p.weight_lbs)),
        Line::from(""),
    ];
    if let Some(class) = app.class {
        let unmet = class.unmet_requirements(&app.score_set());
        let (note, color) = if unmet.is_empty() {
            (format!("  All {} requirements met.", class.label()), Color::Green)
        } else {
            (format!("  {} requirements not met.", unmet.len()), Color::Red)
        };
        sheet.push(Line::from(Span::styled(note, Style::new().fg(color))));
    }

    frame.render_widget(
        Paragraph::new(sheet).block(Block::bordered().title(" Character ")),
        left,
    );

    let mut abilities = ability_lines(app);
    abilities.push(Line::from(""));
    abilities.push(total_line(app));

    frame.render_widget(
        Paragraph::new(abilities).block(
            Block::bordered()
                .title(" Ability Scores ")
                .title_top(
                    Line::from(Span::styled(
                        " ✔ ready — Enter to finish ",
                        Style::new().fg(Color::Green),
                    ))
                    .right_aligned(),
                )
                .title_bottom(aging_line(app)),
        ),
        right,
    );
}

fn character_line(app: &App) -> Line<'static> {
    let class = app.class.map(|c| c.label()).unwrap_or("—");
    Line::from(vec![
        Span::styled("  Class        ", Style::new().fg(MUTED)),
        Span::styled(
            class.to_string(),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn physical_line(app: &App, label: &'static str, value: fn(&crate::character::Physical) -> String) -> Line<'static> {
    let text = match &app.physical {
        Some(physical) => value(physical),
        None => "—".to_string(),
    };
    Line::from(vec![
        Span::styled(label, Style::new().fg(MUTED)),
        Span::styled(text, Style::new().fg(Color::Gray)),
    ])
}

fn aging_line(app: &App) -> Line<'static> {
    let (text, color) = match (&app.physical, app.race) {
        (Some(physical), Some(race)) => {
            let text = physical.aging_label(race);
            if text == "none" {
                (text, MUTED)
            } else {
                (text, Color::Yellow)
            }
        }
        _ => ("—".to_string(), MUTED),
    };
    Line::from(vec![
        Span::styled("  Aging:  ", Style::new().fg(MUTED)),
        Span::styled(text, Style::new().fg(color)),
        Span::raw(" "),
    ])
}

fn level_limit_line(app: &App) -> Line<'static> {
    let limit = match (app.race, app.class) {
        (Some(race), Some(class)) => race
            .level_limit(class)
            .map(|limit| limit.label())
            .unwrap_or_else(|| "class not allowed".to_string()),
        _ => "—".to_string(),
    };
    Line::from(vec![
        Span::styled("  Level limit  ", Style::new().fg(MUTED)),
        Span::styled(limit, Style::new().fg(Color::Gray)),
    ])
}

fn race_line(app: &App) -> Line<'static> {
    let race = app.race.map(|r| r.label()).unwrap_or("—");
    Line::from(vec![
        Span::styled("  Race         ", Style::new().fg(MUTED)),
        Span::styled(
            race.to_string(),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn ability_lines(app: &App) -> Vec<Line<'static>> {
    app.final_assignments()
        .map(|(ability, score)| {
            Line::from(vec![
                Span::styled(format!("  {:<14}", ability.label()), Style::new().fg(LABEL)),
                Span::styled(format!("{score:>2}"), score_style(score)),
                Span::styled(star(score), Style::new().fg(score_style(score).fg.unwrap_or(MUTED))),
            ])
        })
        .collect()
}

fn total_line(app: &App) -> Line<'static> {
    Line::from(vec![
        Span::styled("  Total          ", Style::new().fg(MUTED)),
        Span::styled(
            format!("{:>2}", app.final_total()),
            Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
        ),
    ])
}

fn draw_score_summary(frame: &mut Frame, area: Rect, app: &App) {
    let lines: Vec<Line> = app
        .final_assignments()
        .map(|(ability, score)| {
            Line::from(vec![
                Span::styled(format!(" {:<14}", ability.abbrev()), Style::new().fg(MUTED)),
                Span::styled(format!("{score:>2}"), score_style(score)),
                Span::styled(star(score), Style::new().fg(score_style(score).fg.unwrap_or(MUTED))),
            ])
        })
        .collect();

    let title = match app.race {
        Some(race) => format!(" Scores · {} ", race.label()),
        None => " Scores ".to_string(),
    };
    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(title)),
        area,
    );
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App) {
    let keys = Style::new().fg(ACCENT).add_modifier(Modifier::BOLD);
    let desc = Style::new().fg(MUTED);

    let lines = match app.phase {
        Phase::Menu => vec![
            Line::from(vec![
                Span::styled(" ↑/↓, k/j ", keys),
                Span::styled("move  ", desc),
                Span::styled(" Enter ", keys),
                Span::styled("choose", desc),
            ]),
            Line::from(vec![
                Span::styled(" q/Esc ", keys),
                Span::styled("quit", desc),
            ]),
        ],
        Phase::OrderScores => vec![
            Line::from(vec![
                Span::styled(" ↑/↓, k/j ", keys),
                Span::styled("move  ", desc),
                Span::styled(" ctrl+↑/↓ ", keys),
                Span::styled("(or K/J) shuffle score  ", desc),
                Span::styled(" r ", keys),
                Span::styled("reroll all  ", desc),
            ]),
            Line::from(vec![
                Span::styled(" Enter ", keys),
                Span::styled("accept scores", desc),
                Span::raw("     "),
                Span::styled(" q/Esc ", keys),
                Span::styled("quit", desc),
            ]),
        ],
        Phase::ChooseRace => vec![
            Line::from(vec![
                Span::styled(" ↑/↓, k/j ", keys),
                Span::styled("move  ", desc),
                Span::styled(" Enter ", keys),
                Span::styled("choose race  ", desc),
                Span::styled(" g ", keys),
                Span::styled("random race  ", desc),
            ]),
            Line::from(vec![
                Span::styled(" Backspace ", keys),
                Span::styled("back to scores  ", desc),
                Span::styled(" q/Esc ", keys),
                Span::styled("quit", desc),
            ]),
        ],
        Phase::Review | Phase::Finished => vec![
            Line::from(vec![
                Span::styled(" Enter ", keys),
                Span::styled("finish  ", desc),
                Span::styled(" s ", keys),
                Span::styled("swap sex  ", desc),
                Span::styled(" Backspace ", keys),
                Span::styled("change class  ", desc),
                Span::styled(" q/Esc ", keys),
                Span::styled("quit", desc),
            ]),
        ],
        Phase::ChooseClass => vec![
            Line::from(vec![
                Span::styled(" ↑/↓, k/j ", keys),
                Span::styled("move  ", desc),
                Span::styled(" Enter ", keys),
                Span::styled("choose class  ", desc),
                Span::styled(" Backspace ", keys),
                Span::styled("back to scores  ", desc),
                Span::styled(" q/Esc ", keys),
                Span::styled("quit", desc),
            ]),
        ],
    };

    frame.render_widget(
        Paragraph::new(lines).block(Block::bordered().title(" Keys ")),
        area,
    );
}

/// Bottom title of the class list: the rejection message when there is one,
/// otherwise the legend for the ✔ / ✘ markers.
fn list_footer(app: &App) -> Line<'static> {
    match &app.message {
        Some(message) => Line::from(Span::styled(
            format!(" {message} "),
            Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        None => Line::from(Span::styled(
            " ✔ qualifies   ✘ too low   ✘ race not allowed ",
            Style::new().fg(MUTED),
        )),
    }
}

fn high_score_counter(app: &App, width: u16) -> Line<'static> {
    let high = app.high_score_count();
    let text = if width >= 60 {
        format!(" {high}/6 scores of {HIGH_SCORE}+ (need {MIN_HIGH_SCORES}) ")
    } else {
        format!(" {high}/6 ≥{HIGH_SCORE} ")
    };
    Line::from(Span::styled(
        text,
        Style::new().fg(if high >= MIN_HIGH_SCORES {
            Color::Green
        } else {
            Color::Red
        }),
    ))
    .right_aligned()
}

fn score_style(score: u8) -> Style {
    if score >= HIGH_SCORE {
        Style::new().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else if score <= 8 {
        Style::new().fg(Color::Red)
    } else {
        Style::new().fg(Color::Gray)
    }
}

fn star(score: u8) -> &'static str {
    if score >= HIGH_SCORE { " ★" } else { "" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    /// App past the startup menu, with scores rolled.
    fn started() -> App {
        let mut app = App::new();
        app.accept();
        app
    }

    fn render_text(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| draw(frame, app)).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn menu_lists_every_option() {
        let app = App::new();
        assert_eq!(app.phase, Phase::Menu);
        let text = render_text(&app, 80, 20);
        assert!(text.contains("Step through"), "missing step-through option:\n{text}");
        assert!(text.contains("Random character"), "missing random option");
        assert!(text.contains("Quit"), "missing quit option");
        assert!(text.contains("What would you like to do?"), "missing menu prompt");
    }

    #[test]
    fn menu_renders_on_small_terminals() {
        let app = App::new();
        for (w, h) in [(20, 8), (40, 12), (200, 50)] {
            render_text(&app, w, h);
        }
    }

    #[test]
    fn random_character_review_is_rendered() {
        let mut app = App::new();
        app.random_character();
        assert_eq!(app.phase, Phase::Review);
        let text = render_text(&app, 100, 24);
        assert!(text.contains("Step 4 · Review"), "missing review title");
        assert!(text.contains("Class"), "missing class row");
        assert!(text.contains("Level limit"), "missing level limit row");
    }

    #[test]
    fn renders_all_six_abilities_and_help() {
        let mut app = started();
        app.scores = [16, 15, 14, 13, 12, 16];
        let text = render_text(&app, 80, 24);

        for label in [
            "Strength",
            "Intelligence",
            "Wisdom",
            "Dexterity",
            "Constitution",
            "Charisma",
        ] {
            assert!(text.contains(label), "missing {label} in:\n{text}");
        }
        assert!(text.contains("reroll"), "missing help text");
        assert!(text.contains("2/6 scores of 16+"), "missing high score counter");
    }

    #[test]
    fn renders_in_a_small_terminal_without_panicking() {
        let app = started();
        render_text(&app, 20, 10);
        render_text(&app, 200, 60);
    }

    #[test]
    fn highlight_follows_cursor() {
        let mut app = started();
        app.scores = [10, 11, 12, 13, 14, 15];
        app.cursor = 3;
        let text = render_text(&app, 80, 24);
        assert!(text.contains("▸ Dexterity"));
    }

    /// Walk the app to the class step with a human of the given scores.
    fn class_step(scores: [u8; 6]) -> App {
        let mut app = started();
        app.scores = scores;
        app.accept(); // -> races
        app.race_cursor = Race::ALL.iter().position(|r| *r == Race::Human).unwrap();
        app.accept(); // -> classes
        assert_eq!(app.phase, Phase::ChooseClass);
        app
    }

    #[test]
    fn class_step_lists_every_class_with_markers() {
        let app = class_step([12, 9, 13, 10, 9, 16]);

        let text = render_text(&app, 100, 24);
        for class in Class::ALL {
            assert!(
                text.contains(class.label()),
                "missing {} in:\n{text}",
                class.label()
            );
        }
        assert!(text.contains("✔"), "missing qualified marker");
        assert!(text.contains("✘"), "missing unqualified marker");
        assert!(text.contains("choose class"), "missing step 2 help");
    }

    fn app_in_review() -> App {
        let mut app = class_step([12, 9, 13, 10, 9, 16]);
        app.attempts = 2;
        app.class_cursor = Class::ALL
            .iter()
            .position(|c| *c == Class::Thief)
            .unwrap();
        app.accept();
        assert_eq!(app.phase, Phase::Review);
        app
    }

    #[test]
    fn race_step_shows_adjustments_and_open_classes() {
        let mut app = started();
        app.scores = [16, 12, 13, 15, 9, 16];
        app.accept();
        assert_eq!(app.phase, Phase::ChooseRace);

        let text = render_text(&app, 100, 24);
        for label in ["Human", "Dwarf", "Elf", "Gnome", "Half-Elf", "Halfling", "Half-Orc"] {
            assert!(text.contains(label), "missing {label} in:\n{text}");
        }
        assert!(text.contains("classes open"), "missing open-class count");
        assert!(text.contains("random race"), "missing random hint");
        assert!(text.contains("Con +1"), "missing dwarf adjustment text");
    }

    #[test]
    fn race_step_preview_shows_the_adjusted_scores() {
        let mut app = started();
        app.scores = [16, 12, 13, 15, 9, 16];
        app.accept();
        app.race_cursor = Race::ALL.iter().position(|r| *r == Race::Dwarf).unwrap();
        let text = render_text(&app, 100, 24);
        assert!(text.contains("Scores as Dwarf"), "missing preview title");
        assert!(text.contains("Dex"), "missing preview scores");
    }

    #[test]
    fn review_screen_shows_the_character() {
        let app = app_in_review();
        let text = render_text(&app, 100, 24);

        assert!(text.contains("Step 4 · Review"), "missing step title");
        assert!(text.contains("Human"), "missing chosen race");
        assert!(text.contains("Class"), "missing class label");
        assert!(text.contains("Thief"), "missing chosen class");
        assert!(text.contains("Ability Scores"), "missing score panel");
        assert!(text.contains("Enter to finish"), "missing finish hint");
        assert!(text.contains("Intelligence"), "missing abilities");
    }

    #[test]
    fn review_screen_works_narrow_and_wide() {
        let app = app_in_review();
        for (w, h) in [(50, 14), (76, 18), (120, 30)] {
            let text = render_text(&app, w, h);
            assert!(text.contains("Thief"), "missing class at {w}x{h}");
        }
    }

    #[test]
    fn class_list_shows_racial_level_limits() {
        let mut app = started();
        app.scores = [16, 12, 13, 15, 9, 16];
        app.accept();
        app.race_cursor = Race::ALL.iter().position(|r| *r == Race::Dwarf).unwrap();
        app.accept();

        let text = render_text(&app, 104, 24);
        assert!(text.contains("max 8"), "missing dwarf cleric limit");
        assert!(text.contains("max 9"), "missing dwarf fighter limit");
        assert!(text.contains("max 12"), "missing dwarf thief limit");
        assert!(text.contains("✘ race"), "restricted classes not marked");
    }

    #[test]
    fn human_class_list_has_no_cap_annotations() {
        let app = class_step([16, 12, 15, 15, 12, 16]);
        let text = render_text(&app, 104, 24);
        assert!(!text.contains("max "), "humans should not show a cap:\n{text}");
        assert!(!text.contains("unlimited"), "humans should not show a cap");
    }

    #[test]
    fn human_review_says_unlimited() {
        let mut app = class_step([16, 12, 15, 15, 12, 16]);
        app.class_cursor = Class::ALL.iter().position(|c| *c == Class::Monk).unwrap();
        app.accept();
        assert_eq!(app.phase, Phase::Review);
        let text = render_text(&app, 104, 24);
        assert!(text.contains("Level limit"), "missing limit row");
        assert!(text.contains("unlimited"), "missing unlimited value");
    }

    #[test]
    fn review_shows_the_level_limit() {
        let mut app = started();
        app.scores = [16, 12, 13, 15, 9, 16];
        app.accept();
        app.race_cursor = Race::ALL.iter().position(|r| *r == Race::Dwarf).unwrap();
        app.accept();
        app.class_cursor = Class::ALL.iter().position(|c| *c == Class::Fighter).unwrap();
        app.accept();
        assert_eq!(app.phase, Phase::Review);

        let text = render_text(&app, 104, 24);
        assert!(text.contains("Level limit"), "missing level limit row in:\n{text}");
        assert!(text.contains("max 9"), "missing limit value in:\n{text}");
    }

    #[test]
    fn ineligible_class_message_is_shown() {
        let mut app = class_step([12, 9, 13, 10, 9, 16]);
        app.class_cursor = Class::ALL
            .iter()
            .position(|c| *c == Class::Paladin)
            .unwrap();
        app.accept();
        let text = render_text(&app, 100, 24);
        assert!(text.contains("Cannot be a Paladin"), "missing message in:\n{text}");
    }
}
