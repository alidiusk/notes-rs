use prettytable::{format, Table};

#[macro_export]
macro_rules! build_row {
    ($note:expr, [$(($name:ident, $style:expr)),+]) => {{
        use prettytable::{Row, Cell};

        Row::new(vec![
            $(Cell::new(&$note.$name().clone().to_string()).style_spec($style)),+
        ])
    }};
}

#[macro_export]
macro_rules! build_table {
    ($notes:expr, [$(($name:ident, $style:expr)),+]) => {{
        use crate::build_row;
        use crate::display::new_table;
        use prettytable::{Cell, Row};
        use heck::TitleCase;

        let mut table = new_table();

        table.add_row(Row::new(vec![
            $(Cell::new(&stringify!($name).to_title_case()).style_spec("u")),+
        ]));

        $notes.iter().for_each(|note| {
            table.add_row(build_row!(note,
                [$(($name, $style)),+]
            ));
        });

        table
    }};
}

pub fn new_table() -> Table {
    let mut table = Table::new();

    table.set_format(
        format::FormatBuilder::new()
            .column_separator(' ')
            .separator(
                format::LinePosition::Title,
                format::LineSeparator::new('_', '_', '_', '_'),
            )
            .padding(0, 0)
            .build(),
    );

    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notes::{NoteBuilder, NoteWithId};
    use prettytable::{cell, row};

    #[test]
    fn build_row() {
        let note = &NoteBuilder::new()
            .with_content("Test.")
            .with_desc("Test.")
            .build();
        let note = NoteWithId(0, note);
        let row = row!(note.content().clone(), note.desc().clone());
        let our_row = build_row!(note, [(content, ""), (desc, "")]);

        assert_eq!(row, our_row);
    }

    #[test]
    fn build_table() {
        let note1 = &NoteBuilder::new()
            .with_content("Test.")
            .with_desc("Test.")
            .build();
        let note1 = NoteWithId(0, note1);
        let note2 = &NoteBuilder::new()
            .with_content("Second Test.")
            .with_desc("A description.")
            .build();
        let note2 = NoteWithId(1, note2);

        let mut table = new_table();
        table.add_row(row!(u -> "Content", u -> "Desc"));
        table.add_row(row!(note1.content().clone(), note1.desc().clone()));
        table.add_row(row!(note2.content().clone(), note2.desc().clone()));

        let our_table = build_table!(vec![note1, note2], [(content, ""), (desc, "")]);

        dbg!(&table);
        dbg!(&our_table);

        assert_eq!(table, our_table);
    }
}
