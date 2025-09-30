use tracing::info;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    row: usize,
    col: usize,
    available_rows: usize,
    available_cols: usize,
}

impl Cursor {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            row: 0,
            col: 0,
            available_rows: rows,
            available_cols: cols,
        }
    }

    pub fn advance_n_cols(&mut self, n: usize) {
        info!("Advance cols: cursor at {} {}", self.row, self.col);
        self.col = self
            .col
            .saturating_add(n)
            .min(self.available_cols.saturating_sub(1));
        info!("Advance cols: cursor at {} {}", self.row, self.col);
    }
    pub fn advance_n_rows(&mut self, n: usize) {
        info!("Advance rows: cursor at {} {}", self.row, self.col);
        self.row = self
            .row
            .saturating_add(n)
            .min(self.available_rows.saturating_sub(1));
        info!("Advance rows: cursor at {} {}", self.row, self.col);
    }

    pub fn recede_n_cols(&mut self, n: usize) {
        info!("Recede cols: cursor at {} {}", self.row, self.col);
        self.col = self.col.saturating_sub(n);
        info!("Recede cols: cursor at {} {}", self.row, self.col);
    }
    pub fn recede_n_rows(&mut self, n: usize) {
        info!("Recede rows: cursor at {} {}", self.row, self.col);
        self.row = self.row.saturating_sub(n);
        info!("Recede rows: cursor at {} {}", self.row, self.col);
    }

    pub fn reset_col(&mut self) {
        info!("Reset col: cursor at {} {}", self.row, self.col);
        self.col = 0;
        info!("Reset col: cursor at {} {}", self.row, self.col);
    }

    pub fn reset_row(&mut self) {
        info!("Reset row: cursor at {} {}", self.row, self.col);
        self.row = 0;
        info!("Reset row: cursor at {} {}", self.row, self.col);
    }

    pub fn reset(&mut self) {
        info!("Reset: cursor at {} {}", self.row, self.col);
        self.reset_col();
        self.reset_row();
        info!("Reset: cursor at {} {}", self.row, self.col);
    }

    pub fn last_row_avail(&self) -> bool {
        self.row == self.available_rows.saturating_sub(1)
    }

    pub fn last_col_avail(&self) -> bool {
        self.col == self.available_cols.saturating_sub(1)
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn clamp(&mut self, new_rows_avail: usize, new_cols_avail: usize) {
        info!("Cursor at {} {}", self.row, self.col);
        self.row = self.row.min(new_rows_avail.saturating_sub(1));
        self.col = self.col.min(new_cols_avail.saturating_sub(1));
        self.available_rows = new_rows_avail;
        self.available_cols = new_cols_avail;
        info!("Cursor at {} {}", self.row, self.col);
    }

    pub fn set_col(&mut self, col: usize) {
        info!("Cursor at {} {}", self.row, self.col);
        self.col = col.min(self.available_cols.saturating_sub(1));
        info!("Cursor at {} {}", self.row, self.col);
    }

    pub fn set_row(&mut self, row: usize) {
        info!("Cursor at {} {}", self.row, self.col);
        self.row = row.min(self.available_rows.saturating_sub(1));
        info!("Cursor at {} {}", self.row, self.col);
    }

    pub fn set_position(&mut self, row: usize, col: usize) {
        info!("Cursor at {} {}", self.row, self.col);
        self.set_row(row);
        self.set_col(col);
        info!("Cursor at {} {}", self.row, self.col);
    }
}
