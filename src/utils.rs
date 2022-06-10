/*
 * NCurses utilities
 * -----------------
 */
extern crate ncurses;

use ncurses::*;
use std::cmp::max;

// Geometry
pub const LEFT_BW: i32 = 1;
pub const RIGHT_BW: i32 = 1;
pub const TOP_BW: i32 = 1;
pub const BOT_BW: i32 = 1;

// Colors
pub const FOCUS_COLOR: i16 = 1;
pub const NO_FOCUS_COLOR: i16 = 2;
pub const TITLE_COLOR: i16 = 3;
pub const UNKNOWN_COLOR: i16 = 4;
pub const IN_PLACE_COLOR: i16 = 5;
pub const NOT_IN_PLACE_COLOR: i16 = 6;
pub const NOT_IN_WORD_COLOR: i16 = 7;
pub const NORM_COLOR: i16 = 8;
pub const ERR_COLOR: i16 = 9;
pub const HELP_COLOR: i16 = 10;
pub const YESNO_SEL_COLOR: i16 = 11;
pub const YESNO_NSEL_COLOR: i16 = 12;
pub const DEBUG_COLOR: i16 = 13;

/*
 * Global init
 * -----------
 */
pub fn init() {
    initscr();
    cbreak();
    noecho();
    raw();
    keypad(stdscr(), true);
    start_color();

    init_pair(FOCUS_COLOR, COLOR_RED, COLOR_BLACK);
    init_pair(NO_FOCUS_COLOR, COLOR_CYAN, COLOR_BLACK);
    init_pair(TITLE_COLOR, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(UNKNOWN_COLOR, COLOR_WHITE, COLOR_BLACK);
    init_pair(IN_PLACE_COLOR, COLOR_BLACK, COLOR_GREEN);
    init_pair(NOT_IN_PLACE_COLOR, COLOR_BLACK, COLOR_YELLOW);
    init_pair(NOT_IN_WORD_COLOR, COLOR_WHITE, COLOR_BLACK);
    init_pair(NORM_COLOR, COLOR_WHITE, COLOR_BLACK);
    init_pair(ERR_COLOR, COLOR_WHITE, COLOR_RED);
    init_pair(HELP_COLOR, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(YESNO_SEL_COLOR, COLOR_BLACK, COLOR_CYAN);
    init_pair(YESNO_NSEL_COLOR, COLOR_WHITE, COLOR_BLACK);
    init_pair(DEBUG_COLOR, COLOR_BLACK, COLOR_YELLOW);

    refresh();
}
pub fn end() {
    endwin();
}

/*
 * BASE WINDOW
 * -----------
 */
pub struct Win {
    pub title: String,
    pub w: WINDOW,
    height: i32,
    width: i32,
    #[allow(dead_code)]
    x_offs: i32,
    #[allow(dead_code)]
    y_offs: i32,
    focus: bool,
    xcurs: i32,
    ycurs: i32,
}

impl Win {
    pub fn new(
        title_orig: String,
        focus: bool,
        height: i32,
        width: i32,
        x_offs: i32,
        y_offs: i32,
    ) -> Self {
        let title: String = if title_orig.is_empty() {
            title_orig
        } else {
            format!(" {} ", title_orig)
        };
        let w = newwin(height, width, y_offs, x_offs);
        wattrset(
            w,
            COLOR_PAIR(if focus { FOCUS_COLOR } else { NO_FOCUS_COLOR }),
        );
        box_(w, 0, 0);
        wattrset(w, COLOR_PAIR(TITLE_COLOR));
        mvwprintw(w, 0, (width - title.len() as i32) / 2, title.as_str());
        wrefresh(w);
        Win {
            title,
            w,
            height,
            width,
            x_offs,
            y_offs,
            focus,
            xcurs: LEFT_BW,
            ycurs: TOP_BW,
        }
    }

    #[allow(dead_code)]
    pub fn erase(&mut self) {
        werase(self.w);
        self.xcurs = LEFT_BW;
        self.ycurs = TOP_BW;
        self.box_();
    }

    pub fn box_(&self) {
        wattrset(
            self.w,
            COLOR_PAIR(if self.focus {
                FOCUS_COLOR
            } else {
                NO_FOCUS_COLOR
            }),
        );
        box_(self.w, 0, 0);
        wrefresh(self.w);
    }

    // Focus
    #[allow(dead_code)]
    pub fn set_focus(&mut self, f: bool) {
        self.focus = f;
    }
    #[allow(dead_code)]
    pub fn get_focus(&self) -> bool {
        self.focus
    }

    #[allow(dead_code)]
    pub fn set_cursor(&mut self, y: i32, x: i32) {
        self.xcurs = x;
        self.ycurs = y;
        wmove(self.w, self.ycurs, self.xcurs);
    }
    #[allow(dead_code)]
    pub fn get_cursor(&self) -> (i32, i32) {
        (self.ycurs, self.xcurs)
    }

    pub fn print(&self, clear: bool, mut x: i32, mut y: i32, color: i16, msg: String) {
        if x < 0 {
            x = (self.width - LEFT_BW - RIGHT_BW) / 2;
        }
        x += LEFT_BW;
        if y < 0 {
            y = (self.height - BOT_BW - TOP_BW) / 2;
        }
        y += TOP_BW;

        if clear {
            wmove(self.w, y, 0);
            wclrtoeol(self.w);
        }
        wmove(self.w, y, x);
        wattrset(self.w, COLOR_PAIR(color));
        wprintw(self.w, &msg);
        self.box_();
        wattrset(self.w, COLOR_PAIR(TITLE_COLOR));
        mvwprintw(
            self.w,
            0,
            (self.width - self.title.len() as i32) / 2,
            self.title.as_str(),
        );
        wmove(self.w, self.ycurs, self.xcurs);
        wrefresh(self.w);
    }
}

/*
 * YES/NO WINDOW
 * -------------
 */
const YESNO_HEIGHT: i32 = 8;
const YESNO_WIDTH: i32 = 16;
const MSG_LINE: i32 = 1;
const YESNO_LINE: i32 = 4;
const NO_OFFS: i32 = 9;

pub struct Yesnowin {
    pub title: String,
    pub exit_msg: String,
    pub win: Win,
    #[allow(dead_code)]
    yesno_height: i32,
    yesno_width: i32,
    #[allow(dead_code)]
    msg_line: i32,
    yesno_line: i32,
    no_offs: i32,
    yes: bool,
}

impl Yesnowin {
    pub fn new(title: String, exit_msg: String) -> Self {
        let yesno_height = YESNO_HEIGHT;
        let win = Win::new(
            title.clone(),
            false,
            yesno_height,
            max(title.len() + 4, exit_msg.len()) as i32 + 6,
            (COLS() - LEFT_BW - RIGHT_BW - max(title.len() + 4, exit_msg.len() - 4) as i32) / 2
                + LEFT_BW,
            (LINES() - TOP_BW - BOT_BW - yesno_height) / 2 + TOP_BW,
        );
        Yesnowin {
            title,
            exit_msg,
            win,
            yesno_height,
            yesno_width: YESNO_WIDTH,
            msg_line: MSG_LINE,
            yesno_line: YESNO_LINE,
            no_offs: NO_OFFS,
            yes: true,
        }
    }

    pub fn draw_yesno(&self) {
        wmove(self.win.w, self.yesno_line - TOP_BW, LEFT_BW);
        wclrtoeol(self.win.w);
        if self.yes {
            self.win.print(
                false,
                (self.win.width - self.yesno_width) / 2,
                self.yesno_line,
                YESNO_SEL_COLOR,
                "[ Yes ]".to_string(),
            );
            self.win.print(
                false,
                (self.win.width - self.yesno_width) / 2 + self.no_offs,
                self.yesno_line,
                YESNO_NSEL_COLOR,
                "[ No ]".to_string(),
            );
        } else {
            self.win.print(
                false,
                (self.win.width - self.yesno_width) / 2,
                self.yesno_line,
                YESNO_NSEL_COLOR,
                "[ Yes ]".to_string(),
            );
            self.win.print(
                false,
                (self.win.width - self.yesno_width) / 2 + self.no_offs,
                self.yesno_line,
                YESNO_SEL_COLOR,
                "[ No ]".to_string(),
            );
        }
        wmove(self.win.w, self.win.ycurs, self.win.xcurs);
        wrefresh(self.win.w);
    }

    pub fn run(&mut self) -> bool {
        let p: PANEL = new_panel(self.win.w);
        self.win
            .print(false, 2, 1, NORM_COLOR, self.exit_msg.clone());
        self.draw_yesno();
        show_panel(p);
        loop {
            let ch = getch();
            match ch {
                KEY_LEFT | KEY_RIGHT | 9 => {
                    self.yes = !self.yes;
                    self.draw_yesno();
                }
                89 | 121 => {
                    // 'y' or 'Y'
                    self.yes = true;
                    break;
                }
                78 | 110 => {
                    // 'n' or 'N'
                    self.yes = false;
                    break;
                }
                KEY_ENTER | 10 | 13 => break,
                _ => {}
            }
        }
        hide_panel(p);
        update_panels();
        del_panel(p);
        self.yes
    }
}
pub fn yes_no(title: String, exit_msg: String) -> bool {
    let mut ew: Yesnowin = Yesnowin::new(title, exit_msg);
    ew.run()
}

/*
 * OK MESSAGEBOX
 * -------------
 */
pub struct Msgbox {
    pub title: String,
    pub msg: Vec<String>,
    pub win: Win,
    msg_height: i32,
    msg_width: i32,
    ok_box: bool,
}

impl Msgbox {
    pub fn new(title: String, msg_s: String, ok_box: bool) -> Self {
        let msg: Vec<String> = msg_s.lines().map(|x| x.to_string()).collect();
        let mut msg_height: i32 = msg.len() as i32 + 2 + TOP_BW + BOT_BW;
        if ok_box {
            msg_height += 2;
        }
        let mut msg_width: i32 = title.len() as i32;
        for m in &msg {
            if msg_width < m.len() as i32 {
                msg_width = m.len() as i32;
            }
        }
        msg_width += 2 + LEFT_BW + LEFT_BW;
        let msg_x: i32 = (COLS() - LEFT_BW - RIGHT_BW - msg_width) / 2 + LEFT_BW;
        let msg_y: i32 = (LINES() - TOP_BW - BOT_BW - msg_height) / 2 + TOP_BW;
        let win = Win::new(title.clone(), false, msg_height, msg_width, msg_x, msg_y);
        Msgbox {
            title,
            msg,
            win,
            msg_height,
            msg_width,
            ok_box,
        }
    }

    pub fn run(&mut self) {
        let p: PANEL = new_panel(self.win.w);
        let mut y: i32 = 1;
        for m in &self.msg {
            self.win.print(false, 1, y, NORM_COLOR, m.clone());
            y += 1;
        }
        if self.ok_box {
            self.win.print(
                false,
                (self.win.width - "[ OK ]".len() as i32) / 2,
                y + 1,
                YESNO_SEL_COLOR,
                "[ OK ]".to_string(),
            );
        }
        show_panel(p);
        getch();
        hide_panel(p);
        update_panels();
        del_panel(p);
    }
}
pub fn msg(title: String, msg_s: String, ok_box: bool) {
    let mut msg: Msgbox = Msgbox::new(title, msg_s, ok_box);
    msg.run();
}
