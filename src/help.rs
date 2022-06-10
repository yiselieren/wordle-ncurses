/*
 * NCurses utilities
 * -----------------
 */
extern crate ncurses;

use crate::utils;
use ncurses::*;

/*
 * HELP BOTTOM  WINDOW
 * -------------------
 */
pub struct Help {
    pub height: i32,
    pub width: i32,
    win: WINDOW,
}

impl Help {
    pub fn new(help_str: String, print_legend: bool) -> Self {
        let help: Vec<String> = help_str.lines().map(|x| x.to_string()).collect();
        let legend_lines = if print_legend { 3 } else { 1 };
        let height: i32 = help.len() as i32 + legend_lines;
        let mut width: i32 = 48;
        for m in &help {
            if width < m.len() as i32 {
                width = m.len() as i32;
            }
        }
        let win = newwin(
            height,
            width,
            LINES() - height,
            if width < COLS() {
                (COLS() - width) / 3
            } else {
                0
            },
        );
        wattrset(win, COLOR_PAIR(utils::HELP_COLOR));

        // Print help
        let mut y: i32 = 0;
        for m in &help {
            mvwprintw(win, y, 0, m.as_str());
            y += 1;
        }

        // Print legend
        if print_legend {
            let leg_offset: i32 = 8;
            struct Leg {
                relative_line: i32,
                color: i16,
                legend: String,
            }
            let leg: Vec<Leg> = vec![
                Leg {
                    relative_line: 0,
                    color: utils::HELP_COLOR,
                    legend: "Legend: ".to_string(),
                },
                Leg {
                    relative_line: 0,
                    color: utils::IN_PLACE_COLOR,
                    legend: " X ".to_string(),
                },
                Leg {
                    relative_line: 0,
                    color: utils::HELP_COLOR,
                    legend: " - letter in a correct place".to_string(),
                },
                Leg {
                    relative_line: 1,
                    color: utils::NOT_IN_PLACE_COLOR,
                    legend: " X ".to_string(),
                },
                Leg {
                    relative_line: 1,
                    color: utils::HELP_COLOR,
                    legend: " - letter eixsts in the wrong place ".to_string(),
                },
                Leg {
                    relative_line: 2,
                    color: utils::NOT_IN_WORD_COLOR,
                    legend: " X ".to_string(),
                },
                Leg {
                    relative_line: 2,
                    color: utils::HELP_COLOR,
                    legend: " - letter doesn't exist in the word ".to_string(),
                },
            ];
            let mut x: i32 = 0;
            let mut prev_line = 0;
            for l in leg {
                if l.relative_line != prev_line {
                    x = leg_offset;
                    y += 1;
                    prev_line = l.relative_line;
                }
                wattrset(win, COLOR_PAIR(l.color));
                mvwprintw(win, y, x, l.legend.as_str());
                x += l.legend.len() as i32;
            }
        }
        Help { height, width, win }
    }

    pub fn refresh(&self) {
        wrefresh(self.win);
    }
}

/*
 * DETAILED HELP BIG  WINDOW
 * --------------------------
 */
enum HelpElement {
    Text(String),
    Color(i16),
    Skip(i32),
    NewLine,
    SavePosition(usize),
    RestorePosition(usize),
}

pub fn detailed_help(debug: bool, secret_word: &str) {
    let title = " Help ";
    let mut help_elements: Vec<HelpElement> = vec![
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text("F1        - Display this help screen".to_string()),
        HelpElement::NewLine,
        HelpElement::Text("F10       - Exit".to_string()),
        HelpElement::NewLine,
        HelpElement::Text("Enter     - ".to_string()),
        HelpElement::SavePosition(0),
        HelpElement::Text("Check word if it is completed, no action otherwise".to_string()),
        HelpElement::NewLine,
        HelpElement::Text("Backspace - ".to_string()),
        HelpElement::SavePosition(0),
        HelpElement::Text("Remove last letter in the word and move a focus backward".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Text("works even if word is completed but no checked yet".to_string()),
        HelpElement::NewLine,
        HelpElement::NewLine,
        HelpElement::Text("Legend: ".to_string()),
        HelpElement::SavePosition(0),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Color(utils::FOCUS_COLOR),
        HelpElement::Text("| |".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text(" - ".to_string()),
        HelpElement::SavePosition(1),
        HelpElement::Text("This cell in a focus, any alphanumeric character".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(1),
        HelpElement::Text("pressed on keyboard will be inserterd to the cell".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Color(utils::NO_FOCUS_COLOR),
        HelpElement::Text("|".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text("X".to_string()),
        HelpElement::Color(utils::NO_FOCUS_COLOR),
        HelpElement::Text("|".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text(" - ".to_string()),
        HelpElement::SavePosition(1),
        HelpElement::Text("Letter in the cell but the word checking not performed yet".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(1),
        HelpElement::Text("When word is completed the Enter will check the word.".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Color(utils::IN_PLACE_COLOR),
        HelpElement::Text(" X ".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text(
            " - Letter exists in the word and located in a correct place.".to_string(),
        ),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Color(utils::NOT_IN_PLACE_COLOR),
        HelpElement::Text(" X ".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text(" - Letter exists in the word but located in a wrong place.".to_string()),
        HelpElement::NewLine,
        HelpElement::RestorePosition(0),
        HelpElement::Color(utils::NOT_IN_WORD_COLOR),
        HelpElement::Text(" X ".to_string()),
        HelpElement::Color(utils::NORM_COLOR),
        HelpElement::Text(" - Letter doesn't exist in the word".to_string()),
    ];
    if debug {
        help_elements.push(HelpElement::NewLine);
        help_elements.push(HelpElement::NewLine);
        help_elements.push(HelpElement::Color(utils::HELP_COLOR));
        help_elements.push(HelpElement::Text(format!(
            "       DEBUG MODE: The secret word is \"{}\"",
            secret_word
        )));
    }
    let help_elements_rest: Vec<HelpElement> = vec![
        HelpElement::NewLine,
        HelpElement::NewLine,
        HelpElement::NewLine,
        HelpElement::Color(utils::HELP_COLOR),
        HelpElement::Text("        Press any key to exit help screen".to_string()),
        HelpElement::NewLine,
    ];
    for e in help_elements_rest {
        help_elements.push(e);
    }

    // Caclulate width and height
    let mut height: i32 = 0;
    let mut width: i32 = 0;
    let mut position: i32 = 0;
    let mut max_positions: usize = 0;
    for e in &help_elements {
        match e {
            HelpElement::SavePosition(n) => {
                if max_positions < *n {
                    max_positions = *n;
                }
            }
            HelpElement::RestorePosition(n) => {
                if max_positions < *n {
                    max_positions = *n;
                }
            }
            _ => {}
        }
    }
    let mut old_positions: Vec<usize> = vec![0, max_positions];
    for e in &help_elements {
        match e {
            HelpElement::Color(c) => {}
            HelpElement::Skip(n) => {
                position += n;
            }
            HelpElement::Text(s) => {
                position += s.len() as i32;
                if position > width {
                    width = position;
                }
            }
            HelpElement::NewLine => {
                position = utils::LEFT_BW + 1;
                height += 1;
            }
            HelpElement::SavePosition(n) => {
                old_positions[*n] = position as usize;
            }
            HelpElement::RestorePosition(n) => {
                position = old_positions[*n] as i32;
            }
        }
    }

    // window placement and creation
    width += 2 + utils::LEFT_BW + utils::LEFT_BW;
    height += 2 + utils::TOP_BW + utils::BOT_BW;
    let x: i32 = (COLS() - utils::LEFT_BW - utils::RIGHT_BW - width) / 2 + utils::LEFT_BW;
    let y: i32 = (LINES() - utils::TOP_BW - utils::BOT_BW - height) / 2 + utils::TOP_BW;
    let win: WINDOW = newwin(height, width, y, x);
    wattrset(win, COLOR_PAIR(utils::NO_FOCUS_COLOR));
    box_(win, 0, 0);
    wattrset(win, COLOR_PAIR(utils::TITLE_COLOR));
    mvwprintw(win, 0, (width - title.len() as i32) / 2, title);

    // print help content
    let mut y = utils::TOP_BW + 1;
    let mut position = utils::LEFT_BW + 1;
    for e in &help_elements {
        match e {
            HelpElement::Color(c) => {
                wattrset(win, COLOR_PAIR(*c));
            }
            HelpElement::Skip(n) => {
                position += n;
            }
            HelpElement::Text(s) => {
                mvwprintw(win, y, position, s.as_str());
                position += s.len() as i32;
            }
            HelpElement::NewLine => {
                position = utils::LEFT_BW + 1;
                y += 1;
            }
            HelpElement::SavePosition(n) => {
                old_positions[*n] = position as usize;
            }
            HelpElement::RestorePosition(n) => {
                position = old_positions[*n] as i32;
            }
        }
    }
    wmove(win, utils::TOP_BW, utils::LEFT_BW);
    wrefresh(win);

    let p: PANEL = new_panel(win);
    show_panel(p);
    getch();
    hide_panel(p);
    update_panels();
    del_panel(p);
}
