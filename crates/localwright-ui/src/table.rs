//! Bounded, paginated Iced table views for structured application data.
//!
//! The model owns the table values, while [`paginated_table`] creates widgets for
//! one page only. This bounds view construction; callers remain responsible for
//! loading or streaming the underlying dataset.

use iced::{
    widget::{button, column, container, row, scrollable, text},
    Element, Length,
};
use std::{cmp::Ordering, ops::Range};

/// Maximum number of columns accepted by a table view.
pub const MAX_TABLE_COLUMNS: usize = 128;
/// Maximum number of data rows rendered on a page.
pub const MAX_TABLE_PAGE_SIZE: usize = 100;
const CELL_DISPLAY_CHARS: usize = 256;
const CELL_WIDTH: f32 = 168.0;

/// A scalar cell value rendered without erasing its type.
#[derive(Clone, Debug, PartialEq)]
pub enum CellValue {
    /// Missing value, distinct from an empty string.
    Null,
    Integer(i64),
    UnsignedInteger(u64),
    Float(f64),
    Boolean(bool),
    Text(String),
    /// ISO-formatted date or time value, or another caller-defined temporal display.
    DateTime(String),
    Categorical(String),
}

impl CellValue {
    fn kind(&self) -> Option<u8> {
        match self {
            Self::Null => None,
            Self::Integer(_) => Some(0),
            Self::UnsignedInteger(_) => Some(1),
            Self::Float(_) => Some(2),
            Self::Boolean(_) => Some(3),
            Self::Text(_) => Some(4),
            Self::DateTime(_) => Some(5),
            Self::Categorical(_) => Some(6),
        }
    }

    fn display_value(&self) -> String {
        match self {
            Self::Null => "(null)".into(),
            Self::Integer(value) => value.to_string(),
            Self::UnsignedInteger(value) => value.to_string(),
            Self::Float(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Text(value) | Self::DateTime(value) | Self::Categorical(value) => {
                value.chars().take(CELL_DISPLAY_CHARS).collect()
            }
        }
    }

    fn compare(&self, other: &Self) -> Ordering {
        use CellValue::*;
        match (self, other) {
            (Null, Null) => Ordering::Equal,
            (Null, _) => Ordering::Greater,
            (_, Null) => Ordering::Less,
            (Integer(a), Integer(b)) => a.cmp(b),
            (UnsignedInteger(a), UnsignedInteger(b)) => a.cmp(b),
            (Float(a), Float(b)) => a.total_cmp(b),
            (Boolean(a), Boolean(b)) => a.cmp(b),
            (Text(a), Text(b)) | (DateTime(a), DateTime(b)) | (Categorical(a), Categorical(b)) => {
                a.cmp(b)
            }
            // Construction validates homogeneous non-null values per column.
            _ => Ordering::Equal,
        }
    }
}

/// Stable sort direction for a selected column.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// A typed, row-oriented table model.
#[derive(Clone, Debug, PartialEq)]
pub struct TableModel {
    headers: Vec<String>,
    rows: Vec<Vec<CellValue>>,
}

impl TableModel {
    /// Validate and construct a table. Every row must match the header width, and
    /// each column must contain one non-null scalar type. Null values may appear in
    /// any column; an empty `Text(String::new())` remains a distinct value.
    pub fn new(headers: Vec<String>, rows: Vec<Vec<CellValue>>) -> Result<Self, TableError> {
        if headers.is_empty() || headers.len() > MAX_TABLE_COLUMNS {
            return Err(TableError::InvalidColumnCount);
        }
        if headers.iter().any(|header| header.trim().is_empty())
            || headers.iter().any(|header| header.chars().count() > 256)
        {
            return Err(TableError::InvalidHeader);
        }
        let mut kinds = vec![None; headers.len()];
        for row in &rows {
            if row.len() != headers.len() {
                return Err(TableError::InvalidRowWidth);
            }
            for (column, value) in row.iter().enumerate() {
                if matches!(value, CellValue::Float(number) if !number.is_finite()) {
                    return Err(TableError::NonFiniteFloat);
                }
                if let Some(kind) = value.kind() {
                    match kinds[column] {
                        Some(expected) if expected != kind => {
                            return Err(TableError::MixedColumnTypes(column));
                        }
                        None => kinds[column] = Some(kind),
                        _ => {}
                    }
                }
            }
        }
        Ok(Self { headers, rows })
    }

    /// Column names in display order.
    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Number of rows in the model.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Sort rows by one typed column. Null values remain last in both directions.
    pub fn sort_by(&mut self, column: usize, direction: SortDirection) -> Result<(), TableError> {
        if column >= self.headers.len() {
            return Err(TableError::ColumnOutOfRange);
        }
        self.rows.sort_by(|left, right| {
            let left = &left[column];
            let right = &right[column];
            match (left, right) {
                (CellValue::Null, CellValue::Null) => Ordering::Equal,
                (CellValue::Null, _) => Ordering::Greater,
                (_, CellValue::Null) => Ordering::Less,
                _ => {
                    let order = left.compare(right);
                    if direction == SortDirection::Descending {
                        order.reverse()
                    } else {
                        order
                    }
                }
            }
        });
        Ok(())
    }

    fn page(&self, range: Range<usize>) -> &[Vec<CellValue>] {
        &self.rows[range]
    }
}

/// Errors produced while validating or sorting a [`TableModel`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError {
    InvalidColumnCount,
    InvalidHeader,
    InvalidRowWidth,
    MixedColumnTypes(usize),
    NonFiniteFloat,
    ColumnOutOfRange,
}

impl std::fmt::Display for TableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidColumnCount => {
                write!(formatter, "Table must have 1–{MAX_TABLE_COLUMNS} columns")
            }
            Self::InvalidHeader => write!(
                formatter,
                "Table headers must be nonempty and at most 256 characters"
            ),
            Self::InvalidRowWidth => write!(formatter, "Table row width must match the headers"),
            Self::MixedColumnTypes(column) => write!(
                formatter,
                "Table column {column} contains mixed non-null types"
            ),
            Self::NonFiniteFloat => write!(formatter, "Table floating-point cells must be finite"),
            Self::ColumnOutOfRange => write!(formatter, "Sort column is outside the table"),
        }
    }
}

impl std::error::Error for TableError {}

/// State required to paginate, select rows and expose sort intent to an application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableState {
    page: usize,
    page_size: usize,
    selected_row: Option<usize>,
    sort: Option<(usize, SortDirection)>,
}

impl TableState {
    /// Create state with the requested page size, clamped to the safe range 1–100.
    pub fn new(page_size: usize) -> Self {
        Self {
            page: 0,
            page_size: page_size.clamp(1, MAX_TABLE_PAGE_SIZE),
            selected_row: None,
            sort: None,
        }
    }

    /// The requested page size, never greater than [`MAX_TABLE_PAGE_SIZE`].
    pub fn page_size(&self) -> usize {
        self.page_size
    }

    /// Current zero-based page after clamping to the current data size.
    pub fn current_page(&self, total_rows: usize) -> usize {
        self.page.min(self.page_count(total_rows).saturating_sub(1))
    }

    /// Number of pages; an empty table still has one empty page.
    pub fn page_count(&self, total_rows: usize) -> usize {
        total_rows.div_ceil(self.page_size).max(1)
    }

    /// Row range rendered for the current page.
    pub fn visible_rows(&self, total_rows: usize) -> Range<usize> {
        let start = self.current_page(total_rows).saturating_mul(self.page_size);
        start..start.saturating_add(self.page_size).min(total_rows)
    }

    /// Move to the requested zero-based page, clamped to the data size.
    pub fn set_page(&mut self, page: usize, total_rows: usize) {
        self.page = page.min(self.page_count(total_rows).saturating_sub(1));
    }

    /// Advance one page if a next page exists.
    pub fn next_page(&mut self, total_rows: usize) {
        self.set_page(self.current_page(total_rows).saturating_add(1), total_rows);
    }

    /// Move back one page if possible.
    pub fn previous_page(&mut self, total_rows: usize) {
        self.page = self.current_page(total_rows).saturating_sub(1);
    }

    /// Select an absolute zero-based row index, clearing selection for an invalid index.
    pub fn select_row(&mut self, row: usize, total_rows: usize) {
        self.selected_row = (row < total_rows).then_some(row);
    }

    /// Current absolute selected row, if any.
    pub fn selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    /// Direction the next click on this column should request.
    pub fn next_sort(&self, column: usize) -> SortDirection {
        match self.sort {
            Some((sorted_column, SortDirection::Ascending)) if sorted_column == column => {
                SortDirection::Descending
            }
            _ => SortDirection::Ascending,
        }
    }

    /// Record a sort choice, returning the value for use by the table model.
    pub fn set_sort(&mut self, column: usize, direction: SortDirection) -> (usize, SortDirection) {
        self.sort = Some((column, direction));
        self.page = 0;
        (column, direction)
    }

    fn sort(&self) -> Option<(usize, SortDirection)> {
        self.sort
    }
}

/// Actions emitted by [`paginated_table`]. Applications update their model/state and
/// rebuild the view in response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableAction {
    PageChanged(usize),
    SortBy {
        column: usize,
        direction: SortDirection,
    },
    SelectRow(usize),
}

fn cell<'a, Message: 'a>(value: &CellValue) -> Element<'a, Message> {
    container(text(value.display_value()).size(13))
        .width(Length::Fixed(CELL_WIDTH))
        .padding(4)
        .into()
}

/// Render a native, keyboard-operable, paginated table.
///
/// Only `page_size` rows are converted into Iced widgets. Headers are buttons that
/// request a stable typed sort; row-number buttons request absolute row selection.
/// Cells are rendered with a bounded text preview of at most 256 characters.
pub fn paginated_table<'a, Message: Clone + 'a>(
    table: &'a TableModel,
    state: &'a TableState,
    mut on_action: impl FnMut(TableAction) -> Message + 'a,
) -> Element<'a, Message> {
    let total_rows = table.row_count();
    let current_page = state.current_page(total_rows);
    let pages = state.page_count(total_rows);
    let range = state.visible_rows(total_rows);
    let mut header = row![container(text("Row")).width(Length::Fixed(140.0))].spacing(1);
    for (index, name) in table.headers.iter().enumerate() {
        let label = match state.sort() {
            Some((column, SortDirection::Ascending)) if column == index => format!("{name} ↑"),
            Some((column, SortDirection::Descending)) if column == index => format!("{name} ↓"),
            _ => name.clone(),
        };
        header = header.push(
            button(text(label))
                .width(Length::Fixed(CELL_WIDTH))
                .on_press(on_action(TableAction::SortBy {
                    column: index,
                    direction: state.next_sort(index),
                })),
        );
    }
    let mut lines = column![header].spacing(1);
    for (absolute_row, values) in range.clone().zip(table.page(range)) {
        let row_number = absolute_row + 1;
        let row_label = if state.selected_row() == Some(absolute_row) {
            format!("Row {row_number}, selected")
        } else {
            format!("Select row {row_number}")
        };
        let mut cells = row![button(text(row_label))
            .width(Length::Fixed(140.0))
            .on_press(on_action(TableAction::SelectRow(absolute_row)))]
        .spacing(1);
        for value in values {
            cells = cells.push(cell(value));
        }
        lines = lines.push(cells);
    }
    let first = if total_rows == 0 { 0 } else { range.start + 1 };
    let last = range.end;
    let previous = button(text("Previous page")).on_press_maybe(
        (current_page > 0).then(|| on_action(TableAction::PageChanged(current_page - 1))),
    );
    let next = button(text("Next page")).on_press_maybe(
        (current_page + 1 < pages).then(|| on_action(TableAction::PageChanged(current_page + 1))),
    );
    column![
        text(format!("Rows {first}–{last} of {total_rows}")),
        scrollable(lines)
            .direction(scrollable::Direction::Both {
                vertical: scrollable::Scrollbar::default(),
                horizontal: scrollable::Scrollbar::default(),
            })
            .height(Length::Fixed(360.0)),
        row![
            previous,
            text(format!("Page {} of {pages}", current_page + 1)),
            next
        ]
        .spacing(12)
    ]
    .spacing(8)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbers(rows: usize) -> TableModel {
        TableModel::new(
            vec!["value".into()],
            (0..rows)
                .map(|value| vec![CellValue::Integer(value as i64)])
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn page_size_and_visible_window_are_bounded() {
        let mut state = TableState::new(usize::MAX);
        let table = numbers(10_000);
        assert_eq!(state.page_size(), 100);
        assert_eq!(state.visible_rows(table.row_count()), 0..100);
        state.set_page(42, table.row_count());
        assert_eq!(state.visible_rows(table.row_count()), 4200..4300);
        state.set_page(usize::MAX, table.row_count());
        assert_eq!(state.current_page(table.row_count()), 99);
        assert_eq!(state.visible_rows(table.row_count()), 9900..10_000);
    }

    #[test]
    fn pagination_handles_empty_and_partial_last_pages() {
        let mut state = TableState::new(3);
        assert_eq!(state.visible_rows(0), 0..0);
        let table = numbers(8);
        state.next_page(table.row_count());
        state.next_page(table.row_count());
        assert_eq!(state.visible_rows(table.row_count()), 6..8);
        state.next_page(table.row_count());
        assert_eq!(state.current_page(table.row_count()), 2);
        state.previous_page(table.row_count());
        assert_eq!(state.current_page(table.row_count()), 1);
    }

    #[test]
    fn sort_is_typed_stable_and_keeps_nulls_last_in_both_directions() {
        let mut table = TableModel::new(
            vec!["value".into(), "label".into()],
            vec![
                vec![CellValue::Integer(10), CellValue::Text("first".into())],
                vec![CellValue::Null, CellValue::Text("missing".into())],
                vec![CellValue::Integer(2), CellValue::Text("second".into())],
            ],
        )
        .unwrap();
        table.sort_by(0, SortDirection::Ascending).unwrap();
        assert_eq!(table.rows[0][1], CellValue::Text("second".into()));
        assert_eq!(table.rows[2][1], CellValue::Text("missing".into()));
        table.sort_by(0, SortDirection::Descending).unwrap();
        assert_eq!(table.rows[0][1], CellValue::Text("first".into()));
        assert_eq!(table.rows[2][1], CellValue::Text("missing".into()));
    }

    #[test]
    fn table_rejects_oversized_or_inconsistent_shapes() {
        assert_eq!(
            TableModel::new(vec![], vec![]).unwrap_err(),
            TableError::InvalidColumnCount
        );
        assert_eq!(
            TableModel::new(
                vec!["a".into()],
                vec![vec![CellValue::Integer(1), CellValue::Integer(2)]]
            )
            .unwrap_err(),
            TableError::InvalidRowWidth
        );
        assert_eq!(
            TableModel::new(
                vec!["a".into()],
                vec![
                    vec![CellValue::Integer(1)],
                    vec![CellValue::Text("1".into())]
                ]
            )
            .unwrap_err(),
            TableError::MixedColumnTypes(0)
        );
    }

    #[test]
    fn null_and_empty_text_remain_distinct() {
        let table = TableModel::new(
            vec!["value".into()],
            vec![vec![CellValue::Null], vec![CellValue::Text(String::new())]],
        )
        .unwrap();
        assert_eq!(table.rows[0][0].display_value(), "(null)");
        assert_eq!(table.rows[1][0].display_value(), "");
    }
}
