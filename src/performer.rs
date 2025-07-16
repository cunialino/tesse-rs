use std::sync::{Arc, Mutex};
use vte::{Params, Perform, Parser};
use crate::screen::Screen;

/// A performer that applies parsed bytes to our Screen model
pub struct TerminalPerformer {
    pub screen: Arc<Mutex<Screen>>,
    // Buffer for intermediate OSC/DCS if needed later
    pub osc_buffer: Vec<u8>,
}

impl TerminalPerformer {
    /// Create a new performer given a shared Screen
    pub fn new(screen: Arc<Mutex<Screen>>) -> Self {
        TerminalPerformer {
            screen,
            osc_buffer: Vec::new(),
        }
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        let mut s = self.screen.lock().unwrap();
        s.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        let mut s = self.screen.lock().unwrap();
        match byte {
            b'\n' => s.line_feed(),
            b'\r' => s.cursor_col = 0,
            0x0C    /* FF */ => s.clear(),
            _ => {},
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let mut s = self.screen.lock().unwrap();
        match action {
            // Cursor Position: CSI <row> ; <col> H
            'H' | 'f' => {
                let row = params.first()
                    .and_then(|p| std::str::from_utf8(p).ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                let col = params.get(1)
                    .and_then(|p| std::str::from_utf8(p).ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                s.cursor_row = (row.saturating_sub(1)).min(s.rows - 1);
                s.cursor_col = (col.saturating_sub(1)).min(s.cols - 1);
            }
            // Cursor Up: CSI <n> A
            'A' => {
                let n = params.first()
                    .and_then(|p| std::str::from_utf8(p).ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                s.cursor_row = s.cursor_row.saturating_sub(n);
            }
            // Cursor Down: CSI <n> B
            'B' => {
                let n = params.first()
                    .and_then(|p| std::str::from_utf8(p).ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(1);
                s.cursor_row = (s.cursor_row + n).min(s.rows - 1);
            }
            // Erase Display: CSI 2 J  -> clear screen
            'J' => {
                let mode = params.first()
                    .and_then(|p| std::str::from_utf8(p).ok())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                if mode == 2 {
                    s.clear();
                }
            }
            _ => {}
        }
    }

    // No-ops for now
    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
}

