/// A single cell on the terminal screen, holding a character.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ' }
    }
}

/// A 2D text screen buffer of fixed size, with cursor tracking.
#[derive(Clone, Debug)]
pub struct Screen {
    pub rows: usize,
    pub cols: usize,
    grid: Vec<Vec<Cell>>,
    pub cursor_row: usize,
    pub cursor_col: usize,
}

impl Screen {
    /// Create a new blank screen of given dimensions.
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Screen {
            rows,
            cols,
            grid,
            cursor_row: 0,
            cursor_col: 0,
        }
    }

    /// Clear the entire screen (fill with spaces) and reset cursor.
    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = Cell::default();
            }
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Move cursor to next line, scrolling up if at bottom.
    pub fn line_feed(&mut self) {
        if self.cursor_row + 1 >= self.rows {
            // scroll up
            self.grid.remove(0);
            self.grid.push(vec![Cell::default(); self.cols]);
        } else {
            self.cursor_row += 1;
        }
        self.cursor_col = 0;
    }

    /// Write a character at the cursor, advancing it (with wrap).
    pub fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.line_feed();
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.grid[self.cursor_row][self.cursor_col].ch = c;
            self.cursor_col += 1;
        }
    }

    /// Convert each row into a String for rendering or inspection.
    pub fn to_lines(&self) -> Vec<String> {
        self.grid
            .iter()
            .map(|row| row.iter().map(|cell| cell.ch).collect())
            .collect()
    }

    /// Resize the screen, preserving content where possible.
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        // Adjust rows
        match new_rows.cmp(&self.rows) {
            std::cmp::Ordering::Greater => self.grid.push(vec![Cell::default(); self.cols]),
            std::cmp::Ordering::Less => self.grid.push(vec![Cell::default(); self.cols]),
            _ => (),
        }
        self.rows = new_rows;

        // Adjust cols for each row
        for row in &mut self.grid {
            match new_cols.cmp(&self.cols) {
                std::cmp::Ordering::Greater => row.extend((0..(new_cols - self.cols)).map(|_| Cell::default())),
                std::cmp::Ordering::Less => row.truncate(new_cols),
                _ => ()
            }
        }
        self.cols = new_cols;

        // Clamp cursor
        self.cursor_row = self.cursor_row.min(self.rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(self.cols.saturating_sub(1));
    }
}

// Unit tests for the screen module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_to_lines() {
        let mut s = Screen::new(2, 5);
        s.put_char('H');
        s.put_char('i');
        let lines = s.to_lines();
        assert_eq!(lines, vec!["Hi   ".to_string(), "     ".to_string()]);
    }

    #[test]
    fn test_line_feed_and_scroll() {
        let mut s = Screen::new(2, 3);
        s.put_char('A');
        s.line_feed();
        s.put_char('B');
        s.put_char('C');
        let lines = s.to_lines();
        assert_eq!(lines, vec!["A  ".to_string(), "BC ".to_string()]);
    }

    #[test]
    fn test_clear() {
        let mut s = Screen::new(2, 2);
        s.put_char('X');
        s.line_feed();
        s.put_char('Y');
        s.clear();
        let lines = s.to_lines();
        assert_eq!(lines, vec!["  ".to_string(), "  ".to_string()]);
        assert_eq!(s.cursor_row, 0);
        assert_eq!(s.cursor_col, 0);
    }

    #[test]
    fn test_resize() {
        let mut s = Screen::new(2, 2);
        s.put_char('1');
        s.line_feed();
        s.put_char('2');
        s.resize(3, 3);
        let lines = s.to_lines();
        assert_eq!(
            lines,
            vec!["1  ".to_string(), "2  ".to_string(), "   ".to_string()]
        );
    }
}
