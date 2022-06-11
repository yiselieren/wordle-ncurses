/*
 * Letter box
 * ----------
 */

#![allow(dead_code)]

extern crate ncurses;

use crate::utils;
use ncurses::*;

// Geometry
pub const LB_WIDTH: i32 = 3;
pub const LB_HEIGHT: i32 = 3;
pub enum Role {
    UnknownYet,
    NotInWord,
    NotInPlace,
    InPlace,
}
pub struct Lb {
    pub win: utils::Win,
    border: bool,
    c: char,
    role: Role,
    left_bw: i32,
    right_bw: i32,
    top_bw: i32,
    bot_bw: i32,
}

impl Lb {
    pub fn new(border: bool, focus: bool, x: i32, y: i32) -> Self {
        let win = utils::Win::new("", focus, LB_HEIGHT, LB_WIDTH, x, y);
        Lb {
            win,
            border,
            c: ' ',
            role: Role::UnknownYet,
            left_bw: {
                if border {
                    utils::LEFT_BW
                } else {
                    0
                }
            },
            right_bw: {
                if border {
                    utils::RIGHT_BW
                } else {
                    0
                }
            },
            top_bw: {
                if border {
                    utils::TOP_BW
                } else {
                    0
                }
            },
            bot_bw: {
                if border {
                    utils::BOT_BW
                } else {
                    0
                }
            },
        }
    }

    pub fn set(&mut self, c: char, role: Role) {
        self.c = c.to_ascii_uppercase();
        self.role = role;
        refresh();
    }

    pub fn refresh(&self) {
        let pad: String = format!(
            "{: ^1$}",
            " ",
            LB_WIDTH as usize - self.left_bw as usize - self.right_bw as usize
        );
        wmove(self.win.w, self.top_bw, self.left_bw);
        match self.role {
            Role::UnknownYet => wattrset(self.win.w, COLOR_PAIR(utils::UNKNOWN_COLOR)),
            Role::NotInWord => wattrset(self.win.w, COLOR_PAIR(utils::NOT_IN_WORD_COLOR)),
            Role::NotInPlace => wattrset(self.win.w, COLOR_PAIR(utils::NOT_IN_PLACE_COLOR)),
            Role::InPlace => wattrset(self.win.w, COLOR_PAIR(utils::IN_PLACE_COLOR)),
        };
        for n in 0..(LB_WIDTH - self.top_bw - self.bot_bw) {
            wmove(self.win.w, self.top_bw + n, self.left_bw);
            wprintw(self.win.w, &pad);
        }
        wmove(self.win.w, LB_HEIGHT / 2, LB_WIDTH / 2);
        wprintw(self.win.w, &self.c.to_string());
        if self.border {
            self.win.box_();
        } else if matches!(self.role, Role::UnknownYet) {
            wattrset(
                self.win.w,
                if self.win.get_focus() {
                    COLOR_PAIR(utils::FOCUS_COLOR)
                } else {
                    COLOR_PAIR(utils::NO_FOCUS_COLOR)
                },
            );
            self.win.box_();
        }
        wrefresh(self.win.w);
    }

    pub fn set_role(&mut self, role: Role) {
        self.set(self.c, role);
    }

    pub fn get(&self) -> char {
        self.c
    }
}
