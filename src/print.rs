use std::fmt;

use colored::*;

use crate::models::Note;

pub struct Table {
    column_widths: [usize; 3],
    notes: Vec<Note>,
}

impl Table {
    pub fn new(notes: Vec<Note>) -> Self {
        let column_widths = Table::compute_column_widths(notes.as_slice());

        Table {
            column_widths,
            notes,
        }
    }

    pub fn compute_column_widths(notes: &[Note]) -> [usize; 3] {
        notes.iter().fold([0, 0, 0], |mut arr, note| {
            arr[0] = usize::max(arr[0], note.id_string().len());
            arr[1] = usize::max(arr[1], note.created_string().len());
            arr[2] = usize::max(arr[2], note.content.len());

            arr
        })
    }

    pub fn render_header(&self) -> String {
        format!(
            "{:id$} {:created$} {:content$}",
            "ID".underline(),
            "Created".underline(),
            "Content".underline(),
            id = self.column_widths[0],
            created = self.column_widths[1],
            content = self.column_widths[2],
        )
    }

    pub fn render_row(&self, note: &Note) -> String {
        format!(
            "{:id$} {:created$} {:content$}",
            note.id_string().bold(),
            note.created_string().bold(),
            note.content,
            id = self.column_widths[0],
            created = self.column_widths[1],
            content = self.column_widths[2],
        )
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rows = self
            .notes
            .clone()
            .into_iter()
            .map(|note| self.render_row(&note) + "\n")
            .collect::<String>();
        let header = self.render_header();

        write!(f, "{}\n{}", header, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_column_widths() {
        let notes = vec![
            Note::new(1, "12345678".to_string()),
            Note::new(132, "123456".to_string()),
            Note::new(21, "".to_string()),
        ];

        // [5, 19, 8]
        let expected: [usize; 3] = ["[132]".len(), "2020-02-28 17:05:29".len(), "12345678".len()];

        assert_eq!(expected, Table::compute_column_widths(notes.as_slice()));
    }

    #[test]
    fn render_header() {
        let notes = vec![
            Note::new(1, "12345678".to_string()),
            Note::new(132, "123456".to_string()),
            Note::new(21, "".to_string()),
        ];
        let table = Table::new(notes);

        let expected = format!(
            "{:id$} {:created$} {:content$}",
            "ID".underline(),
            "Created".underline(),
            "Content".underline(),
            id = "[132]".len(),
            created = "2020-02-28 17:05:29".len(),
            content = "12345678".len(),
        );

        assert_eq!(expected, table.render_header());
    }

    // #[test]
    // fn render_row() -> {
    //     let notes = vec![
    //         Note::new("1234".to_string(), "12345678".to_string()),
    //         Note::new("12".to_string(), "123456".to_string()),
    //         Note::new("1234567890".to_string(), "".to_string()),
    //     ];
    //
    //     let table = Table::new(notes);
    // }
}
