use std::fmt;

use colored::*;

pub trait Row {
    fn row(&self) -> Vec<ColoredString>;
    fn header() -> Vec<ColoredString>;
}

pub struct Table<R: Row> {
    column_widths: Vec<usize>,
    rows: Vec<R>,
}

impl<R: Row> Table<R> {
    pub fn new(rows: Vec<R>) -> Self {
        let column_widths = Table::compute_column_widths(rows.as_slice());

        Table {
            column_widths,
            rows,
        }
    }

    /// Computes the column widths for a slice of rows.
    /// NOTE: panics if the header and row vecs are not of equal size!
    pub fn compute_column_widths(rows: &[R]) -> Vec<usize> {
        let header_widths = R::header().iter().map(|s| s.len()).collect();
        rows.iter().fold(header_widths, |widths, row| {
            widths
                .iter()
                .enumerate()
                .map(|(i, w)| usize::max(*w, row.row()[i].len()))
                .collect()
        })
    }

    /// Returns the rendered table as a string.
    pub fn render(&self) -> String {
        // NOTE: string capacity will have to be dynamically increased.
        // Could be optimized.
        let mut render = String::new();

        for (i, s) in R::header().iter().enumerate() {
            render += &format!("{:offset$} ", s, offset = self.column_widths[i]);
        }

        self.rows.iter().for_each(|row| {
            render += "\n";

            row.row().iter().enumerate().for_each(|(i, s)| {
                render += &format!("{:offset$} ", s, offset = self.column_widths[i]);
            });
        });

        render
    }

    pub fn row_width(&self) -> u32 {
        self.column_widths.iter().sum::<usize>() as u32
    }
}

impl<R: Row> fmt::Display for Table<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}
