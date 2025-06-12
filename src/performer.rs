use std::sync::{Arc, Mutex};
use vte::Perform;

pub struct TerminalPerformer {
    buffer: Arc<Mutex<Vec<String>>>,
    pub current_line: String,
    cursor_col: usize,
}

impl TerminalPerformer {
    pub fn new(buffer: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            buffer,
            current_line: String::new(),
            cursor_col: 0,
        }
    }

    pub fn flush_line(&mut self) {
        let line = self.current_line.clone();
        let mut buf = self.buffer.lock().unwrap();
        buf.push(line);
        let buff_len = buf.len();
        if buff_len > 1000 {
            buf.drain(0..buff_len - 1000);
        }
        self.current_line.clear();
        self.cursor_col = 0;
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

