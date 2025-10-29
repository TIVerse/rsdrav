//! Table widget for displaying structured data
//!
//! A table with columns, headers, sorting, and row selection.

use crate::event::{Event, EventResult, KeyCode};
use crate::state::Signal;
use crate::theme::{Color, Modifier, Style};
use crate::view::{Component, EventContext, RenderContext, ViewNode};
use std::sync::Arc;

/// Sort order for table columns
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    fn toggle(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }
}

/// Column definition for a table
#[allow(clippy::type_complexity)]
pub struct Column<T> {
    title: String,
    width: usize,
    render: Arc<dyn Fn(&T) -> String + Send + Sync>,
    sortable: bool,
    /// Sort comparison function (optional, required if sortable)
    sort_key: Option<Arc<dyn Fn(&T) -> String + Send + Sync>>,
}

impl<T> Column<T> {
    /// Create a new column
    pub fn new(title: impl Into<String>, width: usize) -> Self {
        Self {
            title: title.into(),
            width,
            render: Arc::new(|_| String::from("?")),
            sortable: false,
            sort_key: None,
        }
    }

    /// Set the render function for this column
    pub fn render<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        self.render = Arc::new(f);
        self
    }

    /// Make this column sortable with a sort key function
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        // Default: use the render function as sort key
        if self.sort_key.is_none() {
            self.sort_key = Some(self.render.clone());
        }
        self
    }

    /// Set a custom sort key function (automatically makes column sortable)
    pub fn sort_by<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> String + Send + Sync + 'static,
    {
        self.sortable = true;
        self.sort_key = Some(Arc::new(f));
        self
    }
}

/// Table widget for displaying structured data
///
/// ## Example
/// ```no_run
/// use rsdrav::prelude::*;
/// use rsdrav::view::widgets::TableColumn;
///
/// #[derive(Clone)]
/// struct Person {
///     name: String,
///     age: u32,
///     city: String,
/// }
///
/// let data = Signal::new(vec![
///     Person { name: "Alice".into(), age: 30, city: "NYC".into() },
///     Person { name: "Bob".into(), age: 25, city: "LA".into() },
/// ]);
///
/// let selected = Signal::new(Some(0));
///
/// let table = Table::new(data, selected)
///     .column(TableColumn::new("Name", 20).render(|p: &Person| p.name.clone()))
///     .column(TableColumn::new("Age", 5).render(|p: &Person| p.age.to_string()))
///     .column(TableColumn::new("City", 15).render(|p: &Person| p.city.clone()));
/// ```
pub struct Table<T> {
    rows: Signal<Vec<T>>,
    columns: Vec<Column<T>>,
    selected: Signal<Option<usize>>,
    sort: Signal<Option<(usize, SortOrder)>>,
    scroll_offset: usize,
    visible_height: usize,
    style: TableStyle,
}

#[derive(Clone)]
struct TableStyle {
    header: Style,
    normal: Style,
    selected: Style,
    alternating: Style,
}

impl Default for TableStyle {
    fn default() -> Self {
        Self {
            header: Style::default()
                .fg(Color::YELLOW)
                .add_modifier(Modifier::BOLD),
            normal: Style::default(),
            selected: Style::default().bg(Color::BLUE).fg(Color::WHITE),
            alternating: Style::default().bg(Color::rgb(30, 30, 30)),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Table<T> {
    /// Create a new table
    pub fn new(rows: Signal<Vec<T>>, selected: Signal<Option<usize>>) -> Self {
        Self {
            rows,
            columns: Vec::new(),
            selected,
            sort: Signal::new(None),
            scroll_offset: 0,
            visible_height: 10,
            style: TableStyle::default(),
        }
    }

    /// Add a column to the table
    pub fn column(mut self, column: Column<T>) -> Self {
        self.columns.push(column);
        self
    }

    /// Set visible height (rows shown at once)
    pub fn visible_height(mut self, height: usize) -> Self {
        self.visible_height = height;
        self
    }

    /// Format a row into a string with column alignment
    fn format_row(&self, row: &T, is_header: bool) -> String {
        let mut result = String::new();

        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                result.push_str(" │ ");
            }

            let content = if is_header {
                col.title.clone()
            } else {
                (col.render)(row)
            };

            // Truncate or pad to column width
            let formatted = if content.len() > col.width {
                format!("{:.width$}", content, width = col.width - 1) + "…"
            } else {
                format!("{:<width$}", content, width = col.width)
            };

            result.push_str(&formatted);
        }

        result
    }

    /// Render the header row
    fn render_header(&self) -> ViewNode {
        let header_text = self.format_row(&self.rows.get()[0], true);

        // Add sort indicator if column is sorted
        let sort_info = self.sort.get();
        let header_with_sort = if let Some((col_idx, order)) = sort_info {
            if col_idx < self.columns.len() {
                let indicator = match order {
                    SortOrder::Ascending => " ▲",
                    SortOrder::Descending => " ▼",
                };
                format!("{}{}", header_text, indicator)
            } else {
                header_text
            }
        } else {
            header_text
        };

        ViewNode::text_styled(header_with_sort, self.style.header)
    }

    /// Render separator line
    fn render_separator(&self) -> ViewNode {
        let mut sep = String::new();
        for (i, col) in self.columns.iter().enumerate() {
            if i > 0 {
                sep.push_str("─┼─");
            }
            sep.push_str(&"─".repeat(col.width));
        }
        ViewNode::text_styled(sep, Style::default().fg(Color::GRAY))
    }

    /// Select next row
    fn select_next(&mut self) {
        let rows = self.rows.get();
        if rows.is_empty() {
            return;
        }

        let current = self.selected.get();
        let next = match current {
            None => Some(0),
            Some(idx) if idx + 1 < rows.len() => Some(idx + 1),
            Some(idx) => Some(idx),
        };

        self.selected.set(next);
        self.ensure_visible(next.unwrap());
    }

    /// Select previous row
    fn select_prev(&mut self) {
        let rows = self.rows.get();
        if rows.is_empty() {
            return;
        }

        let current = self.selected.get();
        let prev = match current {
            None => Some(rows.len() - 1),
            Some(0) => Some(0),
            Some(idx) => Some(idx - 1),
        };

        self.selected.set(prev);
        self.ensure_visible(prev.unwrap());
    }

    /// Ensure row is visible
    fn ensure_visible(&mut self, index: usize) {
        if index >= self.scroll_offset + self.visible_height {
            self.scroll_offset = index - self.visible_height + 1;
        } else if index < self.scroll_offset {
            self.scroll_offset = index;
        }
    }

    /// Toggle sort on column
    fn toggle_sort(&mut self, col_idx: usize) {
        // Check if column is sortable
        if col_idx >= self.columns.len() || !self.columns[col_idx].sortable {
            return;
        }

        let current = self.sort.get();
        let new_sort = match current {
            None => Some((col_idx, SortOrder::Ascending)),
            Some((idx, order)) if idx == col_idx => {
                // Toggle order on same column
                match order {
                    SortOrder::Ascending => Some((col_idx, SortOrder::Descending)),
                    SortOrder::Descending => None, // Clear sorting
                }
            }
            Some(_) => {
                // Different column, start ascending
                Some((col_idx, SortOrder::Ascending))
            }
        };

        self.sort.set(new_sort);

        // Perform actual sorting
        if let Some((idx, order)) = new_sort {
            self.apply_sort(idx, order);
        }
    }

    /// Apply sorting to the rows
    fn apply_sort(&mut self, col_idx: usize, order: SortOrder) {
        let column = &self.columns[col_idx];

        // Get the sort key function
        let Some(sort_key) = &column.sort_key else {
            return;
        };

        let sort_key = sort_key.clone();

        // Sort the rows
        self.rows.update(|rows| {
            rows.sort_by(|a, b| {
                let key_a = sort_key(a);
                let key_b = sort_key(b);

                match order {
                    SortOrder::Ascending => key_a.cmp(&key_b),
                    SortOrder::Descending => key_b.cmp(&key_a),
                }
            });
        });

        // Reset scroll position after sort
        self.scroll_offset = 0;
    }
}

impl<T: Clone + Send + Sync + 'static> Component for Table<T> {
    fn render(&self, _ctx: &RenderContext) -> ViewNode {
        let rows = self.rows.get();

        if rows.is_empty() {
            return ViewNode::text_styled("(empty table)", Style::default().fg(Color::GRAY));
        }

        let mut children = Vec::new();

        // Header
        children.push(self.render_header());
        children.push(self.render_separator());

        // Visible rows
        let selected_idx = self.selected.get();
        let end = (self.scroll_offset + self.visible_height).min(rows.len());
        let visible_rows = &rows[self.scroll_offset..end];

        for (offset, row) in visible_rows.iter().enumerate() {
            let absolute_idx = self.scroll_offset + offset;
            let is_selected = selected_idx == Some(absolute_idx);
            let is_even = absolute_idx % 2 == 0;

            let row_text = self.format_row(row, false);

            let style = if is_selected {
                self.style.selected
            } else if is_even {
                self.style.alternating
            } else {
                self.style.normal
            };

            let formatted = if is_selected {
                format!("> {}", row_text)
            } else {
                format!("  {}", row_text)
            };

            children.push(ViewNode::text_styled(formatted, style));
        }

        // Scroll indicator
        if rows.len() > self.visible_height {
            let info = format!("  [{}-{} of {}]", self.scroll_offset + 1, end, rows.len());
            children.push(ViewNode::text_styled(
                info,
                Style::default().fg(Color::GRAY),
            ));
        }

        ViewNode::container(children)
    }

    fn handle_event(&mut self, event: &Event, _ctx: &mut EventContext) -> EventResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.select_prev();
                    EventResult::Handled
                }
                KeyCode::Down => {
                    self.select_next();
                    EventResult::Handled
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    // Toggle sort (on first column for now)
                    self.toggle_sort(0);
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, SortOrder, Table};
    use crate::state::Signal;

    #[derive(Clone, Debug)]
    struct TestRow {
        name: String,
        value: i32,
    }

    #[test]
    fn test_table_creation() {
        let data = Signal::new(vec![
            TestRow {
                name: "A".into(),
                value: 1,
            },
            TestRow {
                name: "B".into(),
                value: 2,
            },
        ]);
        let selected = Signal::new(Some(0));

        let table = Table::new(data, selected)
            .column(Column::new("Name", 10).render(|r: &TestRow| r.name.clone()))
            .column(Column::new("Value", 5).render(|r: &TestRow| r.value.to_string()));

        assert_eq!(table.columns.len(), 2);
    }

    #[test]
    fn test_table_navigation() {
        let data = Signal::new(vec![
            TestRow {
                name: "A".into(),
                value: 1,
            },
            TestRow {
                name: "B".into(),
                value: 2,
            },
            TestRow {
                name: "C".into(),
                value: 3,
            },
        ]);
        let selected = Signal::new(Some(0));
        let mut table = Table::new(data, selected.clone())
            .column(Column::new("Name", 10).render(|r: &TestRow| r.name.clone()));

        table.select_next();
        assert_eq!(selected.get(), Some(1));

        table.select_next();
        assert_eq!(selected.get(), Some(2));

        table.select_prev();
        assert_eq!(selected.get(), Some(1));
    }

    #[test]
    fn test_column_render() {
        let data = Signal::new(vec![TestRow {
            name: "Test".into(),
            value: 42,
        }]);
        let selected = Signal::new(None);

        let table = Table::new(data.clone(), selected)
            .column(Column::new("Name", 10).render(|r: &TestRow| r.name.clone()))
            .column(Column::new("Value", 5).render(|r: &TestRow| r.value.to_string()));

        let rows = data.get();
        let formatted = table.format_row(&rows[0], false);
        assert!(formatted.contains("Test"));
        assert!(formatted.contains("42"));
    }

    #[test]
    fn test_sort_toggle() {
        let data = Signal::new(vec![
            TestRow {
                name: "B".into(),
                value: 2,
            },
            TestRow {
                name: "A".into(),
                value: 1,
            },
        ]);
        let selected = Signal::new(None);
        let mut table = Table::new(data.clone(), selected).column(
            Column::new("Name", 10)
                .render(|r: &TestRow| r.name.clone())
                .sortable(),
        );

        assert_eq!(table.sort.get(), None);

        table.toggle_sort(0);
        assert_eq!(table.sort.get(), Some((0, SortOrder::Ascending)));
        // Check data is sorted
        let rows = data.get();
        assert_eq!(rows[0].name, "A");
        assert_eq!(rows[1].name, "B");

        table.toggle_sort(0);
        assert_eq!(table.sort.get(), Some((0, SortOrder::Descending)));
        let rows = data.get();
        assert_eq!(rows[0].name, "B");
        assert_eq!(rows[1].name, "A");

        table.toggle_sort(0);
        assert_eq!(table.sort.get(), None);
    }
}
