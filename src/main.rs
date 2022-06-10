#![allow(unused_variables)]
#![allow(dead_code)]

extern crate ncurses;

use ncurses::*;
use rand::Rng;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
mod help;
mod lb;
mod utils;

use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "data"]
#[prefix = ""]
struct Asset;

use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "wordle-ncurses",
    about = "Ncurses version of the wordle puzzle"
)]
struct Opt {
    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,

    /// Debug mode, the secret word is displayed in a help screen, no dictionary check
    #[structopt(short, long)]
    debug: bool,

    /// Word length
    #[structopt(short = "w", long = "word", default_value = "5")]
    wlen: i32,

    /// Amount of attempts
    #[structopt(short = "t", long = "tries", default_value = "5")]
    tries: i32,
}

struct Line {
    lb: Vec<lb::Lb>,
}
struct Screen {
    lines: Vec<Line>,
    x_focus: usize,
    y_focus: usize,
}
impl Screen {
    pub fn refresh(&self) {
        for line in &self.lines {
            for lb in &line.lb {
                lb.refresh();
            }
        }
        mv(LINES() - 1, 0);
    }
    pub fn right(&mut self) {
        if self.x_focus < self.lines[0].lb.len() - 1 {
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(false);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            self.x_focus += 1;
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(true);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            mv(LINES() - 1, 0);
        }
    }
    pub fn left(&mut self) {
        if self.x_focus > 0 {
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(false);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            self.x_focus -= 1;
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(true);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            mv(LINES() - 1, 0);
        }
    }
    pub fn up(&mut self) {
        if self.y_focus > 0 {
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(false);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            self.y_focus -= 1;
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(true);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            mv(LINES() - 1, 0);
        }
    }
    pub fn down(&mut self) {
        if self.y_focus < self.lines.len() - 1 {
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(false);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            self.y_focus += 1;
            self.lines[self.y_focus].lb[self.x_focus]
                .win
                .set_focus(true);
            self.lines[self.y_focus].lb[self.x_focus].refresh();
            mv(LINES() - 1, 0);
        }
    }
}

fn center(total: i32, len: i32, first: bool) -> i32 {
    if first {
        (total - len) / 3
    } else {
        (total - len) * 2 / 3
    }
}

fn check_word(s: &mut Screen, words: &[String], target_word: &str, debug_mode: bool) -> bool {
    let mut w: String = String::new();
    for lb in &s.lines[s.y_focus].lb {
        w.push(lb.get());
    }
    if words.contains(&w) || debug_mode {
        for (idx, ch) in w.chars().enumerate() {
            if ch == target_word.chars().nth(idx).unwrap() {
                s.lines[s.y_focus].lb[idx].set_role(lb::Role::InPlace);
            } else if target_word.contains(ch) {
                s.lines[s.y_focus].lb[idx].set_role(lb::Role::NotInPlace);
            } else {
                s.lines[s.y_focus].lb[idx].set_role(lb::Role::NotInWord);
            }
        }
        s.refresh();
        if w == *target_word {
            utils::msg(
                "You won!".to_string(),
                format!(
                    "You guessed the right word\n\n          \"{}\"\n\n    From a {}'s attempt!",
                    w,
                    s.y_focus + 1
                ),
                true,
            );
            return true;
        }
    } else {
        utils::msg(
            format!("Word \"{}\"", w),
            "Not in a dictionary".to_string(),
            false,
        );
        s.refresh();
        return false;
    }
    s.refresh();
    if s.y_focus < s.lines.len() - 1 {
        if s.x_focus < s.lines[s.y_focus].lb.len() {
            s.lines[s.y_focus].lb[s.x_focus].win.set_focus(false);
            s.lines[s.y_focus].lb[s.x_focus].refresh();
        }
        s.x_focus = 0;
        s.y_focus += 1;
        s.lines[s.y_focus].lb[s.x_focus].win.set_focus(true);
        s.lines[s.y_focus].lb[s.x_focus].refresh();
        mv(LINES() - 1, 0);
        false
    } else {
        utils::msg(
            "You lost!".to_string(),
            format!("The word is:\n\n    \"{}\"", target_word),
            true,
        );
        true
    }
}

fn main() {
    let opt = Opt::from_args();
    let word_len: i32 = opt.wlen;
    let verbose: bool = opt.verbose;
    let debug: bool = opt.debug;
    let tries: i32 = opt.tries;
    let words: Vec<String> = Vec::new();
    let words1_mtx = Arc::new(Mutex::new(words));
    let words2_mtx = Arc::clone(&words1_mtx);
    let mut secret_word: String = String::new();

    let (tx, rx) = channel();
    let prepare_handle = thread::spawn(move || {
        let tx = tx.clone();
        let words_file = Asset::get("words_alpha.txt").unwrap();
        let mut words = words1_mtx.lock().unwrap();
        *words = std::str::from_utf8(words_file.data.as_ref())
            .unwrap()
            .to_string()
            .split('\n')
            .filter(|&x| x.len() == word_len as usize)
            .map(|x| x.to_uppercase())
            .collect();
        let mut rng = rand::thread_rng();
        tx.send((*words)[rng.gen_range(0, (*words).len())].clone())
            .unwrap();
    });

    // Init ncurses
    utils::init();

    // Help bottom panel
    let help_not_completed: &str =
        "F1 - Help, F10 - Exit, Backspace - Go back\n  Any alphabetic character - insert it";
    let help_completed: &str =
        "F1 - Help, F10 - Exit, Backspace - Go back\n           Enter - Check the word";
    let help_win: help::Help = help::Help::new(help_completed, false);
    help_win.redraw(help_not_completed);

    // Main window
    let mut startx: i32 = center(COLS(), lb::LB_WIDTH * word_len, true);
    let mut starty: i32 = if LINES() > help_win.height {
        center(LINES() - help_win.height, lb::LB_HEIGHT * tries, false)
    } else {
        0
    };
    let mut screen: Screen = Screen {
        lines: Vec::new(),
        x_focus: 0,
        y_focus: 0,
    };
    for y in 0..tries {
        let mut line: Line = Line { lb: Vec::new() };
        for x in 0..word_len {
            line.lb
                .push(lb::Lb::new(false, y == 0 && x == 0, startx, starty));
            startx += lb::LB_WIDTH;
        }
        startx = center(COLS(), lb::LB_WIDTH * word_len, true);
        starty += lb::LB_HEIGHT;
        screen.lines.push(line);
    }
    screen.refresh();

    loop {
        if screen.x_focus >= screen.lines[screen.y_focus].lb.len() {
            help_win.redraw(help_completed);
        } else {
            help_win.redraw(help_not_completed);
        }

        let ch = getch();
        let ch_as_char: char = if ch < 256 {
            std::char::from_u32(ch as u32).unwrap_or('0')
        } else {
            '0'
        };
        if ch == KEY_F(10) {
            if utils::yes_no(
                "Exit confirmation".to_string(),
                "Do you really want to quit from the wordle?".to_string(),
            ) {
                break;
            }
            screen.refresh();
            help_win.refresh();
        } else if ch == KEY_F(1) {
            if secret_word.is_empty() {
                secret_word = rx.recv().unwrap();
            }
            help::detailed_help(debug, &secret_word);
            screen.refresh();
            help_win.refresh();
        } else if ch == KEY_ENTER || ch_as_char == '\n' {
            if screen.x_focus >= screen.lines[screen.y_focus].lb.len() {
                if secret_word.is_empty() {
                    secret_word = rx.recv().unwrap();
                }

                if check_word(
                    &mut screen,
                    &words2_mtx.lock().unwrap(),
                    &secret_word,
                    debug,
                ) {
                    break;
                }
            }
        } else if ch == KEY_BACKSPACE {
            if screen.x_focus > 0 {
                if screen.x_focus < screen.lines[0].lb.len() {
                    screen.lines[screen.y_focus].lb[screen.x_focus].set(' ', lb::Role::UnknownYet);
                    screen.lines[screen.y_focus].lb[screen.x_focus]
                        .win
                        .set_focus(false);
                    screen.lines[screen.y_focus].lb[screen.x_focus].refresh();
                }
                screen.x_focus -= 1;
                screen.lines[screen.y_focus].lb[screen.x_focus].set(' ', lb::Role::UnknownYet);
                screen.lines[screen.y_focus].lb[screen.x_focus]
                    .win
                    .set_focus(true);
                screen.lines[screen.y_focus].lb[screen.x_focus].refresh();
                mv(LINES() - 1, 0);
            }
        } else if ch_as_char.is_alphabetic() {
            if screen.x_focus < screen.lines[screen.y_focus].lb.len() {
                screen.lines[screen.y_focus].lb[screen.x_focus]
                    .set(ch_as_char, lb::Role::UnknownYet);
                screen.lines[screen.y_focus].lb[screen.x_focus]
                    .win
                    .set_focus(false);
                screen.lines[screen.y_focus].lb[screen.x_focus].refresh();
                screen.x_focus += 1;
                if screen.x_focus < screen.lines[screen.y_focus].lb.len() {
                    screen.lines[screen.y_focus].lb[screen.x_focus]
                        .win
                        .set_focus(true);
                    screen.lines[screen.y_focus].lb[screen.x_focus].refresh();
                }
                mv(LINES() - 1, 0);
            }
        } else if debug {
            match ch {
                KEY_LEFT => screen.left(),
                KEY_RIGHT => screen.right(),
                KEY_UP => screen.up(),
                KEY_DOWN => screen.down(),
                _ => {}
            }
        }
    }
    utils::end();
}
