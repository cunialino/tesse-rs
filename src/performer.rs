use std::sync::{Arc, Mutex};
use vte::Perform;

pub struct TerminalPerformer {
    buffer: Arc<Mutex<Vec<String>>>,
    pub current_line: String,
    cursor_col: usize,
    cursor_row: usize,
    pub nrows: u16,
}

impl TerminalPerformer {
    pub fn new(buffer: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            buffer,
            current_line: String::new(),
            cursor_col: 0,
            cursor_row: 0,
            nrows: 0,
        }
    }

    pub fn flush_line(&mut self) {
        let mut buf = self.buffer.lock().unwrap();
        buf.push(std::mem::take(&mut self.current_line));

        let bl = buf.len();
        if buf.len() > 1000 {
            buf.drain(0..bl - 1000);
        }

        self.cursor_col = 0;
    }

    pub fn clear_screen(&mut self) {
        let mut buf = self.buffer.lock().unwrap();

        // Flush current line
        buf.push(std::mem::take(&mut self.current_line));

        // Push enough empty lines to "clear" the visible area
        for _ in 0..self.nrows {
            buf.push("\n".into());
        }

    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        if self.cursor_col >= self.current_line.len() {
            self.current_line.push(c);
        } else {
            self.current_line
                .replace_range(self.cursor_col..self.cursor_col + 1, &c.to_string());
        }
        self.cursor_col += 1;
    }
    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.flush_line(),
            b'\r' => self.cursor_col = 0,
            b'\t' => {
                for _ in 0..4 {
                    self.print(' ');
                }
            }
            b'\x08' => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                    if self.cursor_col < self.current_line.len() {
                        self.current_line.remove(self.cursor_col);
                    }
                }
            }
            0x0C /* FF */ => {
                self.clear_screen();
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            _ => {}
        }
    }
    fn hook(&mut self, _p: &vte::Params, _i: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _b: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn csi_dispatch(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
}
