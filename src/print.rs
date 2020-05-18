use std::fmt;

use colored::*;

use crate::notes::Note;

pub struct Table<'a> {
    column_widths: [usize; 3],
    notes: Vec<(usize, &'a Note)>,
}

impl<'a> Table<'a> {
    pub fn new(notes: Vec<(usize, &'a Note)>) -> Self {
        let column_widths = Table::compute_column_widths(notes.as_slice());

        Table {
            column_widths,
            notes,
        }
    }

    pub fn compute_column_widths(notes: &[(usize, &Note)]) -> [usize; 3] {
        notes.iter().fold([0, 0, 0], |mut arr, (i, note)| {
            arr[0] = usize::max(arr[0], i.to_string().len() + 2);
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

    pub fn render_row(&self, index: usize, note: &Note) -> String {
        let id = "[".to_string() + &index.to_string() + "]";
        format!(
            "{:id$} {:created$} {:content$}",
            id.bold(),
            note.created_string().bold(),
            note.content,
            id = self.column_widths[0],
            created = self.column_widths[1],
            content = self.column_widths[2],
        )
    }
}

impl<'a> fmt::Display for Table<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rows = self
            .notes
            .iter()
            .map(|(i, note)| self.render_row(*i, note) + "\n")
            .collect::<String>();
        let header = self.render_header();

        write!(f, "{}\n{}", header, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::Notes;

    #[test]
    fn compute_column_widths() {
        let notes = Notes::new(vec![
            Note::new("12345678".to_string()),
            Note::new("123456".to_string()),
            Note::new("".to_string()),
        ]);

        // [3, 19, 8]
        let expected: [usize; 3] = ["[1]".len(), "2020-02-28 17:05:29".len(), "12345678".len()];

        assert_eq!(
            expected,
            Table::compute_column_widths(notes.get_all_with_id().unwrap().as_slice())
        );
    }

    #[test]
    fn render_header() {
        let notes = Notes::new(vec![
            Note::new("12345678".to_string()),
            Note::new("123456".to_string()),
            Note::new("".to_string()),
        ]);

        let table = Table::new(notes.get_all_with_id().unwrap());

        let expected = format!(
            "{:id$} {:created$} {:content$}",
            "ID".underline(),
            "Created".underline(),
            "Content".underline(),
            id = "[3]".len(),
            created = "2020-02-28 17:05:29".len(),
            content = "12345678".len(),
        );

        assert_eq!(expected, table.render_header());
    }

    #[test]
    fn render_row() {
        let vec_notes = vec![
            Note::new("12345678".to_string()),
            Note::new("123456".to_string()),
            Note::new("".to_string()),
        ];

        let notes = Notes::new(vec_notes.clone());

        let table = Table::new(notes.get_all_with_id().unwrap());

        let expected = format!(
            "{:3} {:19} {:8}",
            "[1]".bold(),
            notes.get(1).unwrap().created_string().bold(),
            "123456"
        );

        assert_eq!(expected, table.render_row(1, &vec_notes[1]));
    }
}
