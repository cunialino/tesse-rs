use crate::screen::Screen;
use std::sync::{Arc, Mutex};
use tracing::info;
use vte::{Params, Perform};

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
        params: &Params, // params is an iterator, so we need to make it mutable
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let mut s = self.screen.lock().unwrap();
        match action {
            // Cursor Position: CSI <row> ; <col> H
            // CSI Ps ; Ps H
            //   Cursor Position [row;column] (default = [1;1]) (CUH).
            'H' | 'f' => {
                let mut it = params.iter();
                let row = it.next().and_then(|p| p.get(0)).copied().unwrap_or(1) as usize;
                let col = it.next().and_then(|p| p.get(0)).copied().unwrap_or(1) as usize;

                s.cursor_row = row.saturating_sub(1).min(s.rows.saturating_sub(1));
                s.cursor_col = col.saturating_sub(1).min(s.cols.saturating_sub(1));
                info!("moving cursor to position {} {}", row, col);
            }
            'A' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.get(0))
                    .copied()
                    .unwrap_or(1) as usize;
                s.cursor_row = s.cursor_row.saturating_sub(n);
            }
            // Cursor Down: CSI <n> B
            // CSI Ps B
            //   Cursor Down Ps Times (default = 1) (CUD).
            'B' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.get(0))
                    .copied()
                    .unwrap_or(1) as usize;
                s.cursor_row = (s.cursor_row + n).min(s.rows.saturating_sub(1));
            }
            // Erase Display: CSI 2 J  -> clear screen
            // CSI Ps J  —  Erase in Display (ED).
            //   Ps = 0  ⇒  Erase from active position to end of screen (default).
            //   Ps = 1  ⇒  Erase from beginning of screen to active position.
            //   Ps = 2  ⇒  Erase entire screen.
            //   Ps = 3  ⇒  Erase saved lines (if any).
            'J' => {
                let ps = params
                    .iter()
                    .next()
                    .and_then(|p| p.get(0))
                    .copied()
                    .unwrap_or(0) as usize;
                match ps {
                    2 => s.clear(), // Clear entire screen
                    // You can implement other J behaviors here if needed
                    _ => {}
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
