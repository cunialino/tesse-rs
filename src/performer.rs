use crate::{cursor::Cursor, screen::Screen};
use tracing::info;
use vte::{Params, Perform};

pub struct TerminalPerformer {
    pub screen: Screen,
    pub curs: Cursor,
    pub osc_buffer: Vec<u8>,
}

impl TerminalPerformer {
    pub fn new(screen: Screen, curs: Cursor) -> Self {
        TerminalPerformer {
            screen,
            curs,
            osc_buffer: Vec::new(),
        }
    }
    fn clear(&mut self) {
        self.screen.clear();
        self.curs.reset();
    }

    fn line_feed(&mut self) {
        info!("Line feeding");
        if self.curs.last_row_avail() {
            self.screen.scroll_up();
        } else {
            self.curs.advance_n_rows(1);
        }
        self.curs.reset_col();
    }

    fn put_char(&mut self, c: char) {
        if self.curs.last_col_avail() {
            info!("Feeding");
            self.line_feed();
        }
        if !self.curs.last_col_avail() {
            let pos = self.curs.get_position();
            info!("Printing char {} to position {} {}", c, pos.0, pos.1);
            self.screen.char_in_grid(c, pos);
            self.curs.advance_n_cols(1);
        }
    }

    fn resize(&mut self, rows: usize, cols: usize) {
        self.screen.resize(rows, cols);
        self.curs.clamp(rows, cols);
    }
}

impl Perform for TerminalPerformer {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.line_feed(),
            b'\r' => self.curs.reset_col(),
            0x0C    /* FF */ => self.clear(),
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
        match action {
            // CSI Ps ; Ps H
            // The CUP sequence moves the active position to the position specified by the parameters.
            // This sequence has two parameter values, the first specifying the line position and the second
            // specifying the column position. A parameter value of zero or one for the first or second parameter
            // moves the active position to the first line or column in the display, respectively.
            // The default condition with no parameters present is equivalent to a cursor to home action.
            // In the VT100, this control behaves identically with its format effector counterpart, HVP. Editor Function
            //
            // The numbering of lines depends on the state of the Origin Mode (DECOM).
            'H' | 'f' => {
                let mut it = params.iter();
                let row = it.next().and_then(|p| p.get(0)).copied().unwrap_or(1) as usize;
                let col = it.next().and_then(|p| p.get(0)).copied().unwrap_or(1) as usize;

                self.curs
                    .set_position(row.saturating_sub(1), col.saturating_sub(1));
            }
            // ESC [ Pn A
            // default value: 1
            // Moves the active position upward without altering the column position.
            // The number of lines moved is determined by the parameter.
            // A parameter value of zero or one moves the active position one line upward.
            // A parameter value of n moves the active position n lines upward.
            // If an attempt is made to move the cursor above the top margin, the cursor stops at the top margin.
            // Editor Function
            'A' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.get(0))
                    .copied()
                    .unwrap_or(1) as usize;
                self.curs.recede_n_rows(n);
            }
            // CSI Ps B
            //   Cursor Down Ps Times (default = 1) (CUD).
            // The CUD sequence moves the active position downward without altering the column position.
            // The number of lines moved is determined by the parameter.
            // If the parameter value is zero or one, the active position is moved one line downward.
            // If the parameter value is n, the active position is moved n lines downward.
            // In an attempt is made to move the cursor below the bottom margin, the cursor stops at the bottom margin.
            // Editor Function
            'B' => {
                let n = params
                    .iter()
                    .next()
                    .and_then(|p| p.get(0))
                    .copied()
                    .unwrap_or(1) as usize;

                self.curs.advance_n_rows(n);
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
                    2 => self.screen.clear(), // Clear entire screen
                    // You can implement other J behaviors here if needed
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;
    use vte::Parser;

    fn setup_performer(rows: usize, cols: usize) -> TerminalPerformer {
        tracing_subscriber::fmt()
            .with_max_level(Level::DEBUG)
            .init();
        let screen = Screen::new(rows, cols);
        let curs = Cursor::new(rows, cols);
        TerminalPerformer::new(screen, curs)
    }

    #[test]
    fn test_print_single_char() {
        let mut perf = setup_performer(5, 10);
        let mut parser = Parser::new();

        parser.advance(&mut perf, b"A");

        let screen_lock = perf.screen;
        assert_eq!(screen_lock.to_lines()[0], "A         ");
        assert_eq!(perf.curs.get_position(), (0, 1));
    }

    #[test]
    fn test_print_line_wrap() {
        let mut perf = setup_performer(2, 5); // Small screen
        let mut parser = Parser::new();

        parser.advance(&mut perf, b"ABCDEFG");

        let screen_lock = perf.screen;
        assert_eq!(
            screen_lock.to_lines(),
            vec!["ABCDE".to_string(), "FG   ".to_string()]
        );

        assert_eq!(perf.curs.get_position(), (1, 2));
    }

    #[test]
    fn test_execute_line_feed_scrolling() {
        let mut perf = setup_performer(2, 3); // 2 rows, 3 cols
        let mut parser = Parser::new();

        // Fill first line, then second, then third char should trigger scroll
        parser.advance(&mut perf, b"ABC\nDEF\nGHI");

        let screen_lock = perf.screen;
        // A, B, C on first line. \n moves to (1,0)
        // D, E, F on second line. \n moves to (2,0), which scrolls.
        // Screen becomes: [DEF], [empty]. Cursor at (1,0) (due to scroll)
        // G, H, I on now-second line.
        assert_eq!(
            screen_lock.to_lines(),
            vec!["DEF".to_string(), "GHI".to_string()]
        );

        assert_eq!(perf.curs.get_position(), (1, 2));
    }

    #[test]
    fn test_csi_cursor_position() {
        let mut perf = setup_performer(5, 10);
        let mut parser = Parser::new();

        parser.advance(&mut perf, b"\x1B[3;5H"); // Move to row 3, col 5 (0-indexed: 2,4)
        assert_eq!(perf.curs.get_position(), (2, 4));
    }

    #[test]
    fn test_csi_clear_screen() {
        let mut perf = setup_performer(5, 10);
        let mut parser = Parser::new();

        parser.advance(&mut perf, b"Hello, world!\n");
        parser.advance(&mut perf, b"\x1B[3;5H");
        parser.advance(&mut perf, b"X");
        parser.advance(&mut perf, b"\x1B[2J");

        let screen_lock = perf.screen;
        for line in screen_lock.to_lines() {
            assert_eq!(line, "          ");
        }

        assert_eq!(perf.curs.get_position(), (0, 0));
    }

    #[test]
    fn test_csi_cursor_up_clamping() {
        let mut perf = setup_performer(5, 10);
        let mut parser = Parser::new();

        // Initial cursor at (0,0)
        parser.advance(&mut perf, b"\x1B[100A"); // Try to move up 100 lines

        assert_eq!(perf.curs.get_position(), (0, 0));
    }

    #[test]
    fn test_csi_cursor_down_clamping() {
        let mut perf = setup_performer(5, 10);
        let mut parser = Parser::new();

        parser.advance(&mut perf, b"\x1B[100B");

        assert_eq!(perf.curs.get_position(), (4, 0));
    }
}
