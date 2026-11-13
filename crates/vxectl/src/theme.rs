/*
 * Terminal styling, ANSI color formatting, and Unicode card rendering.
 */

use std::io::IsTerminal;

/* Windows ANSI escape code activation via kernel32 SetConsoleMode */
#[cfg(windows)]
pub fn enable_windows_ansi() {
    extern "system" {
        fn GetStdHandle(n_std_handle: u32) -> *mut std::ffi::c_void;
        fn GetConsoleMode(h_console_handle: *mut std::ffi::c_void, lp_mode: *mut u32) -> i32;
        fn SetConsoleMode(h_console_handle: *mut std::ffi::c_void, dw_mode: u32) -> i32;
    }
    unsafe {
        /* STD_OUTPUT_HANDLE = -11i32 as u32 = 0xFFFFFFF5 */
        let handle = GetStdHandle(0xFFFFFFF5);
        let mut mode: u32 = 0;
        if GetConsoleMode(handle, &mut mode) != 0 {
            /* ENABLE_VIRTUAL_TERMINAL_PROCESSING = 0x0004 */
            SetConsoleMode(handle, mode | 0x0004);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub enabled: bool,
}

#[allow(dead_code)]
impl Theme {
    pub fn new(no_color_flag: bool) -> Self {
        let is_term = std::io::stdout().is_terminal();
        let env_no_color = std::env::var_os("NO_COLOR").is_some();
        let enabled = !no_color_flag && !env_no_color && is_term;

        #[cfg(windows)]
        if enabled {
            enable_windows_ansi();
        }

        Self { enabled }
    }

    /* Calculates visible length in terminal monospace columns ignoring ANSI escape codes */
    pub fn visible_len(s: &str) -> usize {
        let mut in_escape = false;
        let mut count = 0;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                count += 1;
            }
        }
        count
    }

    /* Truncates a string to fit within max_visible cells, appending "..." if truncated */
    pub fn truncate_visible(s: &str, max_len: usize) -> String {
        let vlen = Self::visible_len(s);
        if vlen <= max_len {
            return s.to_string();
        }
        let target_len = max_len.saturating_sub(3);
        let mut out = String::new();
        let mut current_visible = 0;
        let mut in_escape = false;

        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
                out.push(c);
            } else if in_escape {
                out.push(c);
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                if current_visible >= target_len {
                    break;
                }
                out.push(c);
                current_visible += 1;
            }
        }
        if in_escape {
            out.push_str("\x1b[0m");
        }
        out.push_str("...");
        out
    }

    /* ANSI Colors */
    pub fn bold(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn dim(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[90m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_dim(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;90m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn cyan(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[36m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_cyan(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;36m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn green(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[32m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_green(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;32m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn yellow(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[33m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_yellow(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;33m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn red(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[31m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_red(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;31m{}\x1b[0m", s) } else { s.to_string() }
    }

    pub fn bold_white(&self, s: &str) -> String {
        if self.enabled { format!("\x1b[1;37m{}\x1b[0m", s) } else { s.to_string() }
    }

    /* Renders a visual 10-bar battery indicator: [████████░░] 85% */
    pub fn battery_bar(&self, pct: u8, charging: bool) -> String {
        let filled = (pct as usize / 10).min(10);
        let empty = 10 - filled;
        let bar_chars = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let colored_bar = if !self.enabled {
            bar_chars
        } else if pct <= 15 {
            format!("\x1b[31m{}\x1b[0m", bar_chars)
        } else if pct <= 35 {
            format!("\x1b[33m{}\x1b[0m", bar_chars)
        } else {
            format!("\x1b[32m{}\x1b[0m", bar_chars)
        };

        let pct_str = if !self.enabled {
            format!("{}%", pct)
        } else if charging {
            format!("\x1b[1;33m{}%\x1b[0m", pct)
        } else if pct <= 15 {
            format!("\x1b[1;31m{}%\x1b[0m", pct)
        } else {
            format!("\x1b[1;32m{}%\x1b[0m", pct)
        };

        let status_str = if charging {
            self.yellow(" (Charging)")
        } else {
            self.dim(" (Discharging)")
        };

        format!("[{}] {}{}", colored_bar, pct_str, status_str)
    }

    /* Status badge for connection mode */
    pub fn mode_badge(&self, mode_str: &str, is_online: bool) -> String {
        if is_online {
            format!("{} {}", self.green("●"), self.bold(mode_str))
        } else {
            format!("{} {}", self.red("●"), self.dim(mode_str))
        }
    }
}

pub enum CardLine {
    Blank,
    Section(String),
    KeyValue { key: String, value: String },
    Raw(String),
}

pub struct Card {
    pub theme: Theme,
    pub width: usize,
    pub title: String,
    pub badge: Option<String>,
    pub lines: Vec<CardLine>,
    pub footer: Option<String>,
}

impl Card {
    pub fn new(theme: Theme, width: usize, title: impl Into<String>) -> Self {
        Self {
            theme,
            width,
            title: title.into(),
            badge: None,
            lines: Vec::new(),
            footer: None,
        }
    }

    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn blank(mut self) -> Self {
        self.lines.push(CardLine::Blank);
        self
    }

    pub fn section(mut self, title: impl Into<String>) -> Self {
        self.lines.push(CardLine::Section(title.into()));
        self
    }

    pub fn key_value(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.lines.push(CardLine::KeyValue {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn raw(mut self, line: impl Into<String>) -> Self {
        self.lines.push(CardLine::Raw(line.into()));
        self
    }

    /* Renders the complete card as a string */
    pub fn render(&self) -> String {
        let w = self.width.max(50);
        let mut out = String::new();

        /* Borders */
        let b_tl = self.theme.dim("╭");
        let b_tr = self.theme.dim("╮");
        let b_bl = self.theme.dim("╰");
        let b_br = self.theme.dim("╯");
        let b_vl = self.theme.dim("│");
        let b_vr = self.theme.dim("│");
        let b_ml = self.theme.dim("├");
        let b_mr = self.theme.dim("┤");

        /* Top Header */
        let t_len = Theme::visible_len(&self.title);
        let b_len = self.badge.as_ref().map(|b| Theme::visible_len(b)).unwrap_or(0);

        let top_line = if let Some(ref badge) = self.badge {
            let fixed = 4 + t_len + 1 + 1 + b_len + 4;
            let dashes = w.saturating_sub(fixed);
            let dash_str = "─".repeat(dashes);
            format!(
                "{}{}{}{}{}{}{}{}{}{}",
                b_tl,
                self.theme.dim("── "),
                self.title,
                self.theme.dim(" "),
                self.theme.dim(&dash_str),
                self.theme.dim(" "),
                badge,
                self.theme.dim(" ──"),
                b_tr,
                "\n"
            )
        } else {
            let fixed = 4 + t_len + 3;
            let dashes = w.saturating_sub(fixed);
            let dash_str = "─".repeat(dashes);
            format!(
                "{}{}{}{}{}{}{}",
                b_tl,
                self.theme.dim("── "),
                self.title,
                self.theme.dim(" "),
                self.theme.dim(&dash_str),
                b_tr,
                "\n"
            )
        };
        out.push_str(&top_line);

        /* Content Lines */
        let inner_w = w.saturating_sub(2);
        let key_pad: usize = 19;

        for line in &self.lines {
            match line {
                CardLine::Blank => {
                    let spaces = " ".repeat(inner_w);
                    out.push_str(&format!("{}{}{}\n", b_vl, spaces, b_vr));
                }
                CardLine::Section(title) => {
                    let s_len = Theme::visible_len(title);
                    let fixed = 3 + s_len + 1 + 1;
                    let dashes = w.saturating_sub(fixed);
                    let dash_str = "─".repeat(dashes);
                    out.push_str(&format!(
                        "{}{}{}{}{}{}\n",
                        b_ml,
                        self.theme.dim("─ "),
                        self.theme.bold_cyan(title),
                        self.theme.dim(" "),
                        self.theme.dim(&dash_str),
                        b_mr
                    ));
                }
                CardLine::KeyValue { key, value } => {
                    let key_len = Theme::visible_len(key);
                    let key_spaces = key_pad.saturating_sub(key_len);
                    let formatted_key = format!("{}{}", self.theme.dim(key), " ".repeat(key_spaces));
                    let formatted_sep = self.theme.dim(" : ");
                    let content_prefix = format!("  {}{}", formatted_key, formatted_sep);
                    let prefix_len = 2 + key_len.max(key_pad) + 3;
                    let avail_val = inner_w.saturating_sub(prefix_len + 1);

                    let val_len = Theme::visible_len(value);
                    let (final_val, final_val_len) = if val_len > avail_val {
                        let trunc = Theme::truncate_visible(value, avail_val);
                        let trunc_len = Theme::visible_len(&trunc);
                        (trunc, trunc_len)
                    } else {
                        (value.clone(), val_len)
                    };

                    let padding = inner_w.saturating_sub(prefix_len + final_val_len);
                    out.push_str(&format!(
                        "{}{}{}{}{}\n",
                        b_vl,
                        content_prefix,
                        final_val,
                        " ".repeat(padding),
                        b_vr
                    ));
                }
                CardLine::Raw(raw) => {
                    let r_len = Theme::visible_len(raw);
                    let avail = inner_w.saturating_sub(3);
                    let (final_raw, final_raw_len) = if r_len > avail {
                        let trunc = Theme::truncate_visible(raw, avail);
                        let trunc_len = Theme::visible_len(&trunc);
                        (trunc, trunc_len)
                    } else {
                        (raw.clone(), r_len)
                    };
                    let padding = inner_w.saturating_sub(2 + final_raw_len);
                    out.push_str(&format!(
                        "{}  {}{}{}\n",
                        b_vl,
                        final_raw,
                        " ".repeat(padding),
                        b_vr
                    ));
                }
            }
        }

        /* Bottom Footer */
        let bot_line = if let Some(ref footer) = self.footer {
            let f_len = Theme::visible_len(footer);
            let fixed = 2 + 1 + f_len + 1 + 5;
            let dashes = w.saturating_sub(fixed);
            let dash_str = "─".repeat(dashes);
            format!(
                "{}{}{}{}{}{}{}\n",
                b_bl,
                self.theme.dim("─"),
                self.theme.dim(&dash_str),
                self.theme.dim(" "),
                self.theme.dim(footer),
                self.theme.dim(" ────"),
                b_br
            )
        } else {
            let dash_str = "─".repeat(w.saturating_sub(2));
            format!(
                "{}{}{}\n",
                b_bl,
                self.theme.dim(&dash_str),
                b_br
            )
        };
        out.push_str(&bot_line);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_len() {
        let plain = "Hello, world!";
        assert_eq!(Theme::visible_len(plain), 13);

        let colored = "\x1b[1;36mHello, world!\x1b[0m";
        assert_eq!(Theme::visible_len(colored), 13);

        let mixed = "\x1b[32m●\x1b[0m \x1b[1mOnline\x1b[0m";
        assert_eq!(Theme::visible_len(mixed), 8);
    }

    #[test]
    fn test_card_width_consistency() {
        let theme = Theme { enabled: false };
        let card = Card::new(theme, 70, "[0] VXE R1 PRO MAX")
            .badge("● Online (2.4 GHz)")
            .section("Performance & Sensor")
            .key_value("Battery Telemetry", "[████████░░] 85% (4.05 V, Discharging)")
            .key_value("Polling Frequency", "1000 Hz (1.0 ms)")
            .key_value("Very Long Key Value That Might Overflow", "This is an extremely long value that should be truncated safely so it does not overflow")
            .footer("vxectl v0.1.0");

        let rendered = card.render();
        for (i, line) in rendered.lines().enumerate() {
            let vlen = Theme::visible_len(line);
            assert_eq!(vlen, 70, "Line {} has incorrect width {}: {:?}", i, vlen, line);
        }
    }
}
