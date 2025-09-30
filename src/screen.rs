use std::ops::Index;

use tracing::info;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
}

impl Default for Cell {
    fn default() -> Self {
        Cell { ch: ' ' }
    }
}

#[derive(Clone, Debug)]
pub struct Screen {
    pub rows: usize,
    pub cols: usize,
    grid: Vec<Vec<Cell>>,
}

impl Screen {
    pub fn new(rows: usize, cols: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Screen { rows, cols, grid }
    }

    pub fn clear(&mut self) {
        for row in &mut self.grid {
            for cell in row.iter_mut() {
                *cell = Cell::default();
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.grid.remove(0);
        self.grid.push(vec![Cell::default(); self.cols]);
    }

    pub fn char_in_grid(&mut self, c: char, pos: (usize, usize)) {
        self.grid[pos.0][pos.1].ch = c;
    }

    /// Convert each row into a String for rendering or inspection.
    pub fn to_lines(&self) -> Vec<String> {
        self.grid
            .iter()
            .map(|row| row.iter().map(|cell| cell.ch).collect())
            .collect()
    }

    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        info!("Resizing to {} {}", new_rows, new_cols);
        match new_rows.cmp(&self.rows) {
            std::cmp::Ordering::Greater => self
                .grid
                .resize_with(new_cols, || vec![Cell::default(); self.cols]),
            std::cmp::Ordering::Less => {
                info!("Dropping rows");
                let dr = self.grid.drain(0..self.rows - new_rows);
                info!("Dropped {:?}", dr.collect::<Vec<_>>());
            }
            _ => {
                info!("Rows are the same as before");
            }
        }
        self.rows = new_rows;

        match new_cols.cmp(&self.cols) {
            std::cmp::Ordering::Greater => {
                for row in &mut self.grid {
                    row.extend((0..(new_cols - self.cols)).map(|_| Cell::default()))
                }
            }
            std::cmp::Ordering::Less => {
                let mut overflow: Vec<_> = Vec::new();

                for row in &mut self.grid {
                    if !overflow.is_empty() {
                        row.splice(0..0, overflow.drain(..));
                    }

                    overflow = row.drain(new_cols..).collect();
                }
            }
            _ => (),
        }
        self.cols = new_cols;
    }
}

#[cfg(test)]
mod tests {
    use tracing::Level;

    use super::*;

    #[test]
    fn test_put_and_to_lines() {
        let mut s = Screen::new(2, 5);
        s.char_in_grid('H', (0, 0));
        s.char_in_grid('i', (0, 1));
        let lines = s.to_lines();
        assert_eq!(lines, vec!["Hi   ".to_string(), "     ".to_string()]);
    }

    #[test]
    fn test_line_feed_and_scroll() {
        let mut s = Screen::new(2, 3);
        s.char_in_grid('A', (0, 0));
        s.char_in_grid('B', (1, 0));
        s.char_in_grid('C', (1, 1));
        let lines = s.to_lines();
        assert_eq!(lines, vec!["A  ".to_string(), "BC ".to_string()]);
    }

    #[test]
    fn test_clear() {
        let mut s = Screen::new(2, 2);
        s.char_in_grid('A', (0, 0));
        s.char_in_grid('B', (1, 0));
        s.clear();
        let lines = s.to_lines();
        assert_eq!(lines, vec!["  ".to_string(), "  ".to_string()]);
    }

    #[test]
    fn test_resize() {
        let mut s = Screen::new(2, 2);
        s.char_in_grid('1', (0, 0));
        s.char_in_grid('2', (1, 0));
        s.resize(3, 3);
        let lines = s.to_lines();
        assert_eq!(
            lines,
            vec!["1  ".to_string(), "2  ".to_string(), "   ".to_string()]
        );
    }

    #[test]
    fn test_resize_shrink() {
        let mut s = Screen::new(2, 3);
        s.char_in_grid('1', (0, 0));
        s.char_in_grid('1', (0, 1));
        s.char_in_grid('1', (0, 2));
        s.resize(2, 2);
        let lines = s.to_lines();
        assert_eq!(lines, vec!["11".to_string(), "1 ".to_string(),]);
    }
}
