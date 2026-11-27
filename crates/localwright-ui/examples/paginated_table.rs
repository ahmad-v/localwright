use iced::{
    widget::{column, text, text_input},
    Element, Length, Task,
};
use localwright_ui::{paginated_table, CellValue, TableAction, TableModel, TableState};

struct TableDemo {
    table: TableModel,
    state: TableState,
    filter: String,
}

#[derive(Clone, Debug)]
enum Message {
    Table(TableAction),
    Filter(String),
}

impl TableDemo {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Filter(filter) => {
                if self.table.filter_contains(&filter).is_ok() {
                    self.filter = filter;
                    self.state.set_page(0, self.table.row_count());
                    self.state.select_row(usize::MAX, self.table.row_count());
                }
            }
            Message::Table(TableAction::PageChanged(page)) => {
                self.state.set_page(page, self.table.row_count());
            }
            Message::Table(TableAction::SortBy { column, direction }) => {
                if self.table.sort_by(column, direction).is_ok() {
                    self.state.set_sort(column, direction);
                }
            }
            Message::Table(TableAction::SelectRow(row)) => {
                self.state.select_row(row, self.table.row_count());
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let selection = self
            .state
            .selected_row()
            .map(|row| format!("Selected source row {}", row + 1))
            .unwrap_or_else(|| "Select a row using its row button".into());
        column![
            text("Localwright bounded table example").size(22),
            text(
                "Only 25 rows are rendered per page; sort typed columns or filter the loaded rows."
            ),
            text_input("Search values in loaded rows", &self.filter).on_input(Message::Filter),
            text(format!("{} rows match", self.table.row_count())),
            text(selection),
            paginated_table(&self.table, &self.state, Message::Table),
        ]
        .padding(20)
        .spacing(12)
        .width(Length::Fill)
        .into()
    }
}

fn main() -> iced::Result {
    iced::application(
        "Localwright paginated table",
        TableDemo::update,
        TableDemo::view,
    )
    .run_with(|| {
        let rows = (0..275)
            .map(|index| {
                vec![
                    CellValue::Integer(index),
                    CellValue::Categorical(format!("Group {}", index % 4 + 1)),
                    if index % 31 == 0 {
                        CellValue::Null
                    } else {
                        CellValue::Float(index as f64 * 0.25)
                    },
                    CellValue::Boolean(index % 2 == 0),
                ]
            })
            .collect();
        (
            TableDemo {
                table: TableModel::new(
                    vec![
                        "row_id".into(),
                        "group".into(),
                        "value".into(),
                        "active".into(),
                    ],
                    rows,
                )
                .expect("the sample schema is valid"),
                state: TableState::new(25),
                filter: String::new(),
            },
            Task::none(),
        )
    })
}
