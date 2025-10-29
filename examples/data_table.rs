//! Data Table Example
//!
//! Demonstrates:
//! - Table widget with multiple columns
//! - Row selection and navigation
//! - Sorting indicators
//! - Styled table with colors
//! - Real-world data display
//!
//! Controls:
//! - ↑/↓ - Navigate rows
//! - s - Toggle sort
//! - r - Refresh data
//! - q - Quit

use rsdrav::prelude::*;

fn main() -> rsdrav::Result<()> {
    App::new()?.root(DataTableDemo::new()).run()
}

#[derive(Clone, Debug)]
struct Employee {
    id: u32,
    name: String,
    department: String,
    salary: f32,
    years: u32,
    status: EmployeeStatus,
}

#[derive(Clone, Debug, PartialEq)]
enum EmployeeStatus {
    Active,
    OnLeave,
    Remote,
}

impl std::fmt::Display for EmployeeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmployeeStatus::Active => write!(f, "Active"),
            EmployeeStatus::OnLeave => write!(f, "On Leave"),
            EmployeeStatus::Remote => write!(f, "Remote"),
        }
    }
}

struct DataTableDemo {
    employees: Signal<Vec<Employee>>,
    selected: Signal<Option<usize>>,
    sort_order: Signal<Option<(usize, SortOrder)>>,
}

impl DataTableDemo {
    fn new() -> Self {
        Self {
            employees: Signal::new(Self::generate_employees()),
            selected: Signal::new(Some(0)),
            sort_order: Signal::new(None),
        }
    }

    fn generate_employees() -> Vec<Employee> {
        vec![
            Employee {
                id: 1001,
                name: "Alice Johnson".into(),
                department: "Engineering".into(),
                salary: 95000.0,
                years: 5,
                status: EmployeeStatus::Active,
            },
            Employee {
                id: 1002,
                name: "Bob Smith".into(),
                department: "Sales".into(),
                salary: 75000.0,
                years: 3,
                status: EmployeeStatus::Remote,
            },
            Employee {
                id: 1003,
                name: "Carol Williams".into(),
                department: "Engineering".into(),
                salary: 105000.0,
                years: 7,
                status: EmployeeStatus::Active,
            },
            Employee {
                id: 1004,
                name: "David Brown".into(),
                department: "Marketing".into(),
                salary: 68000.0,
                years: 2,
                status: EmployeeStatus::OnLeave,
            },
            Employee {
                id: 1005,
                name: "Emma Davis".into(),
                department: "Engineering".into(),
                salary: 98000.0,
                years: 4,
                status: EmployeeStatus::Remote,
            },
            Employee {
                id: 1006,
                name: "Frank Miller".into(),
                department: "HR".into(),
                salary: 72000.0,
                years: 6,
                status: EmployeeStatus::Active,
            },
            Employee {
                id: 1007,
                name: "Grace Lee".into(),
                department: "Sales".into(),
                salary: 82000.0,
                years: 4,
                status: EmployeeStatus::Active,
            },
            Employee {
                id: 1008,
                name: "Henry Chen".into(),
                department: "Engineering".into(),
                salary: 110000.0,
                years: 8,
                status: EmployeeStatus::Remote,
            },
        ]
    }

    fn get_selected_employee(&self) -> Option<Employee> {
        let employees = self.employees.get();
        let idx = self.selected.get()?;
        employees.get(idx).cloned()
    }
}

impl Component for DataTableDemo {
    fn render(&self, ctx: &RenderContext) -> ViewNode {
        // Title
        let title = Text::new("=== Employee Directory ===")
            .fg(Color::YELLOW)
            .add_modifier(Modifier::BOLD);

        // Statistics
        let employees = self.employees.get();
        let total = employees.len();
        let active = employees
            .iter()
            .filter(|e| e.status == EmployeeStatus::Active)
            .count();
        let remote = employees
            .iter()
            .filter(|e| e.status == EmployeeStatus::Remote)
            .count();

        let stats = VStack::new()
            .push(Text::new(format!("Total Employees: {}", total)).fg(Color::CYAN))
            .push(
                Text::new(format!(
                    "Active: {}  Remote: {}  On Leave: {}",
                    active,
                    remote,
                    total - active - remote
                ))
                .fg(Color::GRAY),
            );

        // Employee table
        let table = Table::new(self.employees.clone(), self.selected.clone())
            .column(TableColumn::new("ID", 8).render(|e: &Employee| e.id.to_string()))
            .column(TableColumn::new("Name", 18).render(|e: &Employee| e.name.clone()))
            .column(TableColumn::new("Department", 14).render(|e: &Employee| e.department.clone()))
            .column(
                TableColumn::new("Salary", 12).render(|e: &Employee| format!("${:.0}", e.salary)),
            )
            .column(TableColumn::new("Years", 7).render(|e: &Employee| e.years.to_string()))
            .column(TableColumn::new("Status", 10).render(|e: &Employee| e.status.to_string()))
            .visible_height(10);

        let table_panel = Panel::new()
            .title("Employees")
            .border_style(Style::default().fg(Color::BLUE))
            .child(table);

        // Selected employee details
        let details = if let Some(emp) = self.get_selected_employee() {
            Panel::new()
                .title("Selected Employee")
                .border_style(Style::default().fg(Color::GREEN))
                .child(
                    VStack::new()
                        .push(Text::new(format!("ID: {}", emp.id)).fg(Color::CYAN))
                        .push(Text::new(format!("Name: {}", emp.name)).fg(Color::WHITE))
                        .push(
                            Text::new(format!("Department: {}", emp.department)).fg(Color::YELLOW),
                        )
                        .push(Text::new(format!("Salary: ${:.2}", emp.salary)).fg(Color::GREEN))
                        .push(Text::new(format!("Years: {}", emp.years)).fg(Color::GRAY))
                        .push(
                            Text::new(format!("Status: {}", emp.status)).fg(match emp.status {
                                EmployeeStatus::Active => Color::GREEN,
                                EmployeeStatus::OnLeave => Color::YELLOW,
                                EmployeeStatus::Remote => Color::BLUE,
                            }),
                        ),
                )
        } else {
            Panel::new()
                .title("Selected Employee")
                .child(Text::new("No selection").fg(Color::GRAY))
        };

        // Controls
        let controls = VStack::new()
            .push(Text::new(""))
            .push(Text::new("Controls:").fg(Color::YELLOW))
            .push(Text::new("  ↑/↓ - Navigate employees").fg(Color::GRAY))
            .push(Text::new("  s - Toggle sort").fg(Color::GRAY))
            .push(Text::new("  r - Refresh data").fg(Color::GRAY))
            .push(Text::new("  q - Quit").fg(Color::GRAY));

        // Layout
        VStack::new()
            .gap(1)
            .push(title)
            .push(stats)
            .push(Text::new(""))
            .push(table_panel)
            .push(Text::new(""))
            .push(details)
            .push(controls)
            .render(ctx)
    }

    fn handle_event(&mut self, event: &Event, ctx: &mut EventContext) -> EventResult {
        // Let table handle navigation
        let mut table = Table::new(self.employees.clone(), self.selected.clone());
        let result = table.handle_event(event, ctx);
        if result != EventResult::Ignored {
            return result;
        }

        // Handle custom commands
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('r') | KeyCode::Char('R') => {
                    // Refresh (in real app, would fetch from API)
                    self.employees.set(Self::generate_employees());
                    EventResult::Handled
                }
                _ => EventResult::Ignored,
            },
            _ => EventResult::Ignored,
        }
    }
}
