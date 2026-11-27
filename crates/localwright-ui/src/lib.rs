#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
mod commands;
mod preferences;
mod shortcuts;
mod table;
pub use commands::{CommandDescriptor, CommandMenu, WorkbenchCommand, COMMAND_REGISTRY};
use iced::{
    keyboard::{key::Named, Key, Modifiers},
    theme::Palette,
    widget::{
        button, column, container, pane_grid, pick_list, row, scrollable, text, text_editor,
        text_input, toggler,
    },
    Border, Color, Element, Length, Subscription, Task, Theme,
};
use localwright_core::{Error, ErrorKind, Handler, Job, Operation, Outcome, Request, Result};
use preferences::{app_preferences_path, Preferences};
pub use shortcuts::{ShortcutBinding, ShortcutCommand, ShortcutKey, ShortcutRegistry};
use std::{fmt, path::PathBuf};
pub use table::{
    paginated_table, CellValue, SortDirection, TableAction, TableError, TableModel, TableState,
    MAX_TABLE_COLUMNS, MAX_TABLE_PAGE_SIZE,
};

/// A theme name followed by the RGB background, text, and accent colors.
pub type ThemePreset = (&'static str, [u8; 3], [u8; 3], [u8; 3]);

/// Initial presets for client-area colors.
pub const THEMES: &[ThemePreset] = &[
    ("Modern Light", [250, 250, 250], [25, 25, 28], [29, 90, 155]),
    (
        "Modern Dark",
        [30, 31, 36],
        [238, 238, 240],
        [125, 180, 255],
    ),
    ("Graphite", [45, 46, 48], [245, 245, 245], [171, 190, 205]),
    ("Midnight", [12, 21, 40], [235, 240, 255], [125, 160, 255]),
    ("Classic 95", [192, 192, 192], [0, 0, 0], [0, 0, 128]),
    ("Classic 2000", [212, 208, 200], [0, 0, 0], [10, 36, 106]),
    ("Luna", [236, 233, 216], [25, 25, 25], [36, 79, 163]),
    ("Silver", [224, 226, 234], [20, 20, 25], [70, 85, 130]),
    ("Aqua", [239, 243, 247], [20, 35, 50], [30, 100, 175]),
    ("Workstation", [216, 216, 208], [20, 20, 20], [54, 73, 96]),
    ("Retro Beige", [241, 231, 205], [55, 40, 20], [120, 70, 30]),
    ("Paper", [255, 252, 242], [35, 32, 29], [95, 55, 30]),
    ("Nordic", [46, 52, 64], [236, 239, 244], [136, 192, 208]),
    (
        "Solarized Light",
        [253, 246, 227],
        [60, 75, 80],
        [0, 91, 137],
    ),
    (
        "Solarized Dark",
        [0, 43, 54],
        [238, 232, 213],
        [95, 185, 220],
    ),
    ("Phosphor Green", [5, 14, 7], [130, 255, 145], [60, 180, 85]),
    (
        "Amber Terminal",
        [20, 13, 4],
        [255, 200, 95],
        [217, 145, 50],
    ),
    ("Monochrome", [240, 240, 240], [0, 0, 0], [75, 75, 75]),
    (
        "High Contrast Light",
        [255, 255, 255],
        [0, 0, 0],
        [0, 0, 140],
    ),
    (
        "High Contrast Dark",
        [0, 0, 0],
        [255, 255, 255],
        [255, 230, 0],
    ),
];
fn rgb(c: [u8; 3]) -> Color {
    Color::from_rgb8(c[0], c[1], c[2])
}
fn preset(name: &str) -> Theme {
    let p = THEMES.iter().find(|p| p.0 == name).unwrap_or(&THEMES[0]);
    Theme::custom(
        p.0.to_owned(),
        Palette {
            background: rgb(p.1),
            text: rgb(p.2),
            primary: rgb(p.3),
            success: Color::from_rgb8(30, 140, 80),
            danger: Color::from_rgb8(200, 50, 50),
        },
    )
}

fn control_border(theme: &Theme, focused: bool) -> Border {
    let radius = match theme.to_string().as_str() {
        "Classic 95"
        | "Classic 2000"
        | "Workstation"
        | "Retro Beige"
        | "Phosphor Green"
        | "Amber Terminal"
        | "Monochrome"
        | "High Contrast Light"
        | "High Contrast Dark" => 0.0,
        "Luna" | "Silver" | "Aqua" => 8.0,
        _ => 4.0,
    };
    Border {
        radius: radius.into(),
        width: if focused { 3.0 } else { 1.0 },
        color: theme.extended_palette().background.base.text,
    }
}

const MAX_PREVIEW_TABLE_COLUMNS: usize = 256;
const MAX_PREVIEW_TABLE_ROWS: usize = 100;
const MAX_PREVIEW_TABLE_CELLS: usize = 8192;
const PREVIEW_TABLE_CELL_WIDTH: f32 = 168.0;

struct PreviewTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    total_rows: usize,
    warnings: String,
}

fn preview_table(report: &str) -> Option<PreviewTable> {
    if report.len() > 128 * 1024 {
        return None;
    }
    let mut stream = serde_json::Deserializer::from_str(report).into_iter::<serde_json::Value>();
    let value = stream.next()?.ok()?;
    let consumed = stream.byte_offset();
    let object = value.as_object()?;
    let header_values = object.get("headers")?.as_array()?;
    if header_values.is_empty() || header_values.len() > MAX_PREVIEW_TABLE_COLUMNS {
        return None;
    }
    let headers = header_values
        .iter()
        .map(|value| value.as_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?;
    let preview_values = object.get("preview")?.as_array()?;
    if preview_values.len() > MAX_PREVIEW_TABLE_ROWS
        || preview_values.len().saturating_mul(headers.len()) > MAX_PREVIEW_TABLE_CELLS
    {
        return None;
    }
    let mut rows = Vec::with_capacity(preview_values.len());
    for row in preview_values {
        let row = row.as_array()?;
        if row.len() != headers.len() {
            return None;
        }
        let cells = row
            .iter()
            .map(|value| match value {
                serde_json::Value::Null => Some(String::new()),
                serde_json::Value::String(value) => Some(value.chars().take(512).collect()),
                serde_json::Value::Number(value) => Some(value.to_string()),
                serde_json::Value::Bool(value) => Some(value.to_string()),
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => None,
            })
            .collect::<Option<Vec<_>>>()?;
        rows.push(cells);
    }
    let total_rows = object
        .get("rows")
        .and_then(serde_json::Value::as_u64)
        .and_then(|rows| usize::try_from(rows).ok())
        .unwrap_or(rows.len());
    let warnings = report[consumed..].trim().chars().take(2048).collect();
    Some(PreviewTable {
        headers,
        rows,
        total_rows,
        warnings,
    })
}

fn preview_table_cell(value: String) -> Element<'static, Message> {
    container(text(value).size(13))
        .width(Length::Fixed(PREVIEW_TABLE_CELL_WIDTH))
        .padding(4)
        .into()
}

fn preview_table_view(table: PreviewTable) -> Element<'static, Message> {
    let mut header_cells = Vec::with_capacity(table.headers.len() + 1);
    header_cells.push(preview_table_cell("#".into()));
    header_cells.extend(table.headers.into_iter().map(preview_table_cell));
    let mut lines: Vec<Element<'static, Message>> = vec![row(header_cells).spacing(1).into()];
    for (index, values) in table.rows.into_iter().enumerate() {
        let mut cells = Vec::with_capacity(values.len() + 1);
        cells.push(preview_table_cell((index + 1).to_string()));
        cells.extend(values.into_iter().map(preview_table_cell));
        lines.push(row(cells).spacing(1).into());
    }
    let mut content = column![text(format!(
        "Showing {} of {} rows",
        lines.len().saturating_sub(1),
        table.total_rows
    ))]
    .spacing(6);
    if !table.warnings.is_empty() {
        content = content.push(text(table.warnings).wrapping(text::Wrapping::WordOrGlyph));
    }
    content
        .push(
            scrollable(column(lines).spacing(1))
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::default(),
                    horizontal: scrollable::Scrollbar::default(),
                })
                .height(Length::Fixed(360.0)),
        )
        .into()
}

#[derive(Clone, Copy)]
struct OperationChoice(&'static Operation);

impl PartialEq for OperationChoice {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl fmt::Display for OperationChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.label)
    }
}

const SCALES: [u16; 7] = [80, 100, 125, 150, 200, 250, 300];

struct State {
    ops: &'static [Operation],
    handler: Handler,
    projects: Vec<Project>,
    active_project: u64,
    next_project_id: u64,
    preferences: Preferences,
    preferences_path: Option<PathBuf>,
    preferences_error: Option<String>,
    job: Option<Job>,
    job_project: Option<u64>,
    status: String,
    panes: pane_grid::State<WorkspacePane>,
    workspace_split: pane_grid::Split,
    active_menu: Option<CommandMenu>,
}

const MAX_PROJECT_TABS: usize = 32;

/// Session-local project state. Each tab owns its independent operation inputs and results.
struct Project {
    id: u64,
    title: String,
    selected: String,
    input: String,
    input_selection: Option<PathBuf>,
    output: String,
    output_selection: Option<PathBuf>,
    parameter: String,
    editor: text_editor::Content,
    preview: String,
    workspace_tab: WorkspaceTab,
}

impl Project {
    fn new(id: u64, ops: &[Operation]) -> Self {
        Self {
            id,
            title: format!("Project {id}"),
            selected: ops
                .first()
                .map(|operation| operation.id)
                .unwrap_or("")
                .into(),
            input: String::new(),
            input_selection: None,
            output: String::new(),
            output_selection: None,
            parameter: String::new(),
            editor: text_editor::Content::new(),
            preview: String::new(),
            workspace_tab: WorkspaceTab::Result,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspacePane {
    Controls,
    Workspace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkspaceTab {
    Result,
    Draft,
}
#[derive(Debug, Clone)]
enum Message {
    NewProject,
    ActivateProject(u64),
    CloseProject(u64),
    Select(&'static str),
    Input(String),
    Output(String),
    Parameter(String),
    Edit(text_editor::Action),
    Theme(String),
    Scale(u16),
    ToolbarVisible(bool),
    ShortcutKey(ShortcutCommand, String),
    PickInput,
    InputPicked(u64, Option<PathBuf>),
    PickOutput,
    OutputPicked(u64, Option<PathBuf>),
    Run,
    Cancel,
    KeyPressed(Key, Modifiers),
    Done(u64, Result<Outcome>),
    Resize(pane_grid::ResizeEvent),
    WorkspaceTab(WorkspaceTab),
    Menu(CommandMenu),
    Command(WorkbenchCommand),
}
impl State {
    fn active_project(&self) -> &Project {
        self.projects
            .iter()
            .find(|project| project.id == self.active_project)
            .expect("the workbench always has an active project")
    }

    fn active_project_mut(&mut self) -> &mut Project {
        self.projects
            .iter_mut()
            .find(|project| project.id == self.active_project)
            .expect("the workbench always has an active project")
    }

    fn add_project(&mut self) {
        if self.projects.len() >= MAX_PROJECT_TABS {
            self.status = format!("At most {MAX_PROJECT_TABS} project tabs can be open");
            return;
        }
        let id = self.next_project_id;
        let Some(next_id) = id.checked_add(1) else {
            self.status = "Project identifier space exhausted; restart the workbench".into();
            return;
        };
        self.next_project_id = next_id;
        self.projects.push(Project::new(id, self.ops));
        self.active_project = id;
        self.status = format!("Opened Project {id}");
    }

    fn close_project(&mut self, id: u64) {
        if self.projects.len() <= 1 {
            self.status = "Keep at least one project tab open".into();
            return;
        }
        let Some(index) = self.projects.iter().position(|project| project.id == id) else {
            return;
        };
        if self.job_project == Some(id) {
            if let Some(job) = &self.job {
                job.cancel();
            }
            self.status = "Project closed; waiting for its operation to stop".into();
        }
        self.projects.remove(index);
        if self.active_project == id {
            self.active_project = self.projects[index.min(self.projects.len() - 1)].id;
        }
    }

    fn cycle_project(&mut self, reverse: bool) {
        if self.projects.len() < 2 {
            return;
        }
        let current = self
            .projects
            .iter()
            .position(|project| project.id == self.active_project)
            .unwrap_or(0);
        let next = if reverse {
            (current + self.projects.len() - 1) % self.projects.len()
        } else {
            (current + 1) % self.projects.len()
        };
        self.active_project = self.projects[next].id;
    }

    fn resize_controls(&mut self, delta: i8) {
        let current = i16::from(self.preferences.panel_ratio);
        let next = (current + i16::from(delta)).clamp(20, 70) as u8;
        if next == self.preferences.panel_ratio {
            self.status = format!("Controls pane is at its {next}% limit");
            return;
        }
        self.preferences.panel_ratio = next;
        self.panes
            .resize(self.workspace_split, f32::from(next) / 100.0);
        self.save_preferences();
        self.status = format!("Controls pane width: {next}%");
    }

    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::NewProject => self.add_project(),
            Message::ActivateProject(id) => {
                if self.projects.iter().any(|project| project.id == id) {
                    self.active_project = id;
                }
            }
            Message::CloseProject(id) => self.close_project(id),
            Message::Select(v) => self.active_project_mut().selected = v.into(),
            Message::Input(v) => {
                let project = self.active_project_mut();
                project.input = v;
                project.input_selection = None;
            }
            Message::Output(v) => {
                let project = self.active_project_mut();
                project.output = v;
                project.output_selection = None;
            }
            Message::Parameter(v) => self.active_project_mut().parameter = v,
            Message::Edit(a) => self.active_project_mut().editor.perform(a),
            Message::Theme(v) => {
                self.preferences.theme = v;
                self.save_preferences();
            }
            Message::Scale(v) => {
                self.preferences.scale = v;
                self.save_preferences();
            }
            Message::ToolbarVisible(visible) => {
                self.preferences.toolbar_visible = visible;
                self.save_preferences();
            }
            Message::ShortcutKey(command, label) => {
                if let (Some(key), Some(mut binding)) = (
                    ShortcutKey::from_label(&label),
                    self.preferences.shortcuts.binding(command).cloned(),
                ) {
                    binding.key = key;
                    match self.preferences.shortcuts.set(command, binding) {
                        Ok(()) => self.save_preferences(),
                        Err(error) => {
                            self.preferences_error =
                                Some(format!("Shortcut was not changed: {error}"));
                        }
                    }
                }
            }
            Message::PickInput => {
                let project_id = self.active_project;
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select input file")
                            .pick_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    move |path| Message::InputPicked(project_id, path),
                );
            }
            Message::InputPicked(id, path) => {
                if let Some(project) = self.projects.iter_mut().find(|project| project.id == id) {
                    apply_path_selection(&mut project.input, &mut project.input_selection, path);
                }
            }
            Message::PickOutput => {
                let project_id = self.active_project;
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Choose output file")
                            .save_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    move |path| Message::OutputPicked(project_id, path),
                );
            }
            Message::OutputPicked(id, path) => {
                if let Some(project) = self.projects.iter_mut().find(|project| project.id == id) {
                    apply_path_selection(&mut project.output, &mut project.output_selection, path);
                }
            }
            Message::Cancel => {
                if let Some(j) = &self.job {
                    j.cancel();
                    self.status = "Cancellation requested; waiting for operation".into();
                }
            }
            Message::KeyPressed(key, modifiers) => {
                if modifiers.control() && matches!(key.as_ref(), Key::Named(Named::Tab)) {
                    self.cycle_project(modifiers.shift());
                    return Task::none();
                }
                match self.preferences.shortcuts.resolve(&key, modifiers) {
                    Some(ShortcutCommand::Run) => {
                        return self.update(Message::Command(WorkbenchCommand::Run));
                    }
                    Some(ShortcutCommand::Cancel) => {
                        return self.update(Message::Command(WorkbenchCommand::Cancel));
                    }
                    Some(ShortcutCommand::NarrowControls) => self.resize_controls(-5),
                    Some(ShortcutCommand::WidenControls) => self.resize_controls(5),
                    None => {}
                }
            }
            Message::Run => {
                if self.job.is_some() {
                    return Task::none();
                }
                let project_id = self.active_project;
                let project = self.active_project();
                let request = Request {
                    operation: project.selected.clone(),
                    input: project
                        .input_selection
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&project.input)),
                    output: project
                        .output_selection
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&project.output)),
                    parameter: project.parameter.clone(),
                    draft: project.editor.text(),
                };
                let job = Job::new();
                self.job = Some(job.clone());
                self.job_project = Some(project_id);
                self.status = "Running operation".into();
                self.active_project_mut().preview.clear();
                let handler = self.handler;
                let (tx, rx) = futures::channel::oneshot::channel();
                let spawn = std::thread::Builder::new()
                    .name("localwright-job".into())
                    .spawn(move || {
                        let _ = tx.send(handler(request, job));
                    });
                if let Err(e) = spawn {
                    self.job = None;
                    self.job_project = None;
                    self.status = e.to_string();
                    return Task::none();
                }
                return Task::perform(
                    async move {
                        rx.await.unwrap_or_else(|_| {
                            Err(Error::new(
                                ErrorKind::Internal,
                                "Worker stopped unexpectedly",
                            ))
                        })
                    },
                    move |result| Message::Done(project_id, result),
                );
            }
            Message::Done(project_id, result) => {
                self.job = None;
                self.job_project = None;
                match result {
                    Ok(o) => {
                        self.status = if let Some(p) = o.written {
                            format!("Wrote {}", p.display())
                        } else {
                            "Finished".into()
                        };
                        let Some(project) = self.projects.iter_mut().find(|p| p.id == project_id)
                        else {
                            return Task::none();
                        };
                        project.preview = o.preview;
                        for warning in o.warnings {
                            project.preview.push_str(&format!("\nWarning: {warning}"));
                        }
                        if let Some(d) = o.draft {
                            project.editor = text_editor::Content::with_text(&d);
                        }
                    }
                    Err(e) => {
                        self.status = format!("Error: {:?}", e.kind);
                        if let Some(project) = self.projects.iter_mut().find(|p| p.id == project_id)
                        {
                            project.preview = e.message;
                        }
                    }
                }
            }
            Message::Resize(event) => {
                let ratio = normalized_panel_ratio(event.ratio, self.preferences.panel_ratio);
                self.panes.resize(event.split, ratio);
                let ratio_percent = (ratio * 100.0).round() as u8;
                if self.preferences.panel_ratio != ratio_percent {
                    self.preferences.panel_ratio = ratio_percent;
                    self.save_preferences();
                }
            }
            Message::WorkspaceTab(tab) => self.active_project_mut().workspace_tab = tab,
            Message::Menu(menu) => {
                self.active_menu = if self.active_menu == Some(menu) {
                    None
                } else {
                    Some(menu)
                };
            }
            Message::Command(command) => {
                self.active_menu = None;
                return match command {
                    WorkbenchCommand::OpenInput => self.update(Message::PickInput),
                    WorkbenchCommand::ChooseOutput => self.update(Message::PickOutput),
                    WorkbenchCommand::Run => self.update(Message::Run),
                    WorkbenchCommand::Cancel => self.update(Message::Cancel),
                    WorkbenchCommand::ShowResults => {
                        self.update(Message::WorkspaceTab(WorkspaceTab::Result))
                    }
                    WorkbenchCommand::ShowDraft => {
                        self.update(Message::WorkspaceTab(WorkspaceTab::Draft))
                    }
                    WorkbenchCommand::ShowShortcuts => {
                        self.status = self.preferences.shortcuts.summary();
                        Task::none()
                    }
                    WorkbenchCommand::ToggleToolbar => {
                        self.preferences.toolbar_visible = !self.preferences.toolbar_visible;
                        self.save_preferences();
                        Task::none()
                    }
                };
            }
        }
        Task::none()
    }
    fn save_preferences(&mut self) {
        if let Some(path) = &self.preferences_path {
            self.preferences_error = self
                .preferences
                .save(path)
                .err()
                .map(|error| format!("Could not save workbench preferences: {error}"));
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let project = self.active_project();
        let status = match &self.preferences_error {
            Some(error) => format!("{}\n{error}", self.status),
            None => self.status.clone(),
        };
        let ops = self.ops.iter().map(OperationChoice).collect::<Vec<_>>();
        let themes = THEMES.iter().map(|t| t.0.to_owned()).collect::<Vec<_>>();
        let shortcut_keys = shortcut_key_labels();
        let selected = ops.iter().copied().find(|o| o.0.id == project.selected);
        let hint = selected.map(|o| o.0.hint).unwrap_or("");
        let menu_bar = row(CommandMenu::all()
            .iter()
            .copied()
            .map(|menu| button(menu.label()).on_press(Message::Menu(menu)).into())
            .collect::<Vec<Element<'_, Message>>>())
        .spacing(4);
        let menu_items = self.active_menu.map(|menu| {
            row(COMMAND_REGISTRY
                .iter()
                .filter(|descriptor| descriptor.menu == menu)
                .map(|descriptor| {
                    button(descriptor.label)
                        .on_press(Message::Command(descriptor.command))
                        .into()
                })
                .collect::<Vec<Element<'_, Message>>>())
            .spacing(6)
        });
        let toolbar = row(COMMAND_REGISTRY
            .iter()
            .filter(|descriptor| descriptor.toolbar)
            .map(|descriptor| -> Element<'_, Message> {
                let button = button(descriptor.label);
                match descriptor.command {
                    WorkbenchCommand::Run if self.job.is_none() && selected.is_some() => {
                        button.on_press(Message::Command(descriptor.command)).into()
                    }
                    WorkbenchCommand::Cancel if self.job.is_some() => {
                        button.on_press(Message::Command(descriptor.command)).into()
                    }
                    WorkbenchCommand::Run | WorkbenchCommand::Cancel => button.into(),
                    _ => button.on_press(Message::Command(descriptor.command)).into(),
                }
            })
            .collect::<Vec<Element<'_, Message>>>())
        .spacing(8)
        .wrap();
        let mut project_tabs = row![text("Projects")]
            .spacing(4)
            .align_y(iced::Alignment::Center);
        for tab in &self.projects {
            let active = tab.id == self.active_project;
            let label = if active {
                format!("Active: {}", tab.title)
            } else {
                tab.title.clone()
            };
            project_tabs =
                project_tabs.push(button(text(label)).on_press(Message::ActivateProject(tab.id)));
            let close = button(text(format!("Close {}", tab.title)));
            project_tabs = project_tabs.push(if self.projects.len() > 1 {
                close.on_press(Message::CloseProject(tab.id))
            } else {
                close
            });
        }
        let add_project = button("+ Project");
        project_tabs = project_tabs.push(if self.projects.len() < MAX_PROJECT_TABS {
            add_project.on_press(Message::NewProject)
        } else {
            add_project
        });
        let content = pane_grid::PaneGrid::new(&self.panes, |_, panel, _| {
            let body: Element<'_, Message> = match panel {
                WorkspacePane::Controls => scrollable(
                    container(
                        column![
                            row![
                                row![
                                    text("Operation"),
                                    pick_list(ops.clone(), selected, |choice| Message::Select(
                                        choice.0.id
                                    ))
                                    .style(picker_style)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                                row![
                                    text("Theme"),
                                    pick_list(
                                        themes.clone(),
                                        Some(self.preferences.theme.clone()),
                                        Message::Theme
                                    )
                                    .style(picker_style)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                                row![
                                    text("UI scale (%)"),
                                    pick_list(SCALES, Some(self.preferences.scale), Message::Scale)
                                        .style(picker_style)
                                ]
                                .spacing(8)
                                .align_y(iced::Alignment::Center),
                            ]
                            .spacing(16)
                            .wrap(),
                            toggler(self.preferences.toolbar_visible)
                                .label("Show toolbar")
                                .on_toggle(Message::ToolbarVisible),
                            text(hint),
                            text("Input path"),
                            row![
                                text_input("Path to local input", &project.input)
                                    .on_input(Message::Input)
                                    .padding(8)
                                    .style(input_style),
                                button("Browse…")
                                    .on_press(Message::Command(WorkbenchCommand::OpenInput))
                            ]
                            .spacing(8)
                            .wrap(),
                            text("New output path"),
                            row![
                                text_input("Output path", &project.output)
                                    .on_input(Message::Output)
                                    .padding(8)
                                    .style(input_style),
                                button("Save as…")
                                    .on_press(Message::Command(WorkbenchCommand::ChooseOutput))
                            ]
                            .spacing(8)
                            .wrap(),
                            text("Operation parameter"),
                            text_input("See operation description", &project.parameter)
                                .on_input(Message::Parameter)
                                .padding(8)
                                .style(input_style),
                            row![
                                text("Run shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::Run)
                                        .map(|binding| binding.key.label()),
                                    |label| Message::ShortcutKey(ShortcutCommand::Run, label)
                                )
                                .style(picker_style),
                                text("Cancel shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::Cancel)
                                        .map(|binding| binding.key.label()),
                                    |label| Message::ShortcutKey(ShortcutCommand::Cancel, label)
                                )
                                .style(picker_style),
                                text("Narrow controls shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::NarrowControls)
                                        .map(|binding| binding.key.label()),
                                    |label| {
                                        Message::ShortcutKey(ShortcutCommand::NarrowControls, label)
                                    }
                                )
                                .style(picker_style),
                                text("Widen controls shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::WidenControls)
                                        .map(|binding| binding.key.label()),
                                    |label| {
                                        Message::ShortcutKey(ShortcutCommand::WidenControls, label)
                                    }
                                )
                                .style(picker_style),
                            ]
                            .spacing(8)
                            .align_y(iced::Alignment::Center)
                            .wrap(),
                            text(self.preferences.shortcuts.summary()),
                        ]
                        .spacing(8),
                    )
                    .width(Length::Fill)
                    .padding(12),
                )
                .into(),
                WorkspacePane::Workspace => {
                    let tabs = row![
                        button("Results").on_press(Message::WorkspaceTab(WorkspaceTab::Result)),
                        button("Draft / transcript")
                            .on_press(Message::WorkspaceTab(WorkspaceTab::Draft)),
                    ]
                    .spacing(8);
                    let panel = match project.workspace_tab {
                        WorkspaceTab::Result => {
                            let result: Element<'_, Message> = preview_table(&project.preview)
                                .map(preview_table_view)
                                .unwrap_or_else(|| {
                                    text(&project.preview)
                                        .wrapping(text::Wrapping::WordOrGlyph)
                                        .into()
                                });
                            column![text("Result and warnings"), result].spacing(8)
                        }
                        WorkspaceTab::Draft => column![text("Editable draft / transcript"), {
                            let editor = text_editor(&project.editor).height(Length::Fill).style(
                                |theme, status| {
                                    let mut style = text_editor::default(theme, status);
                                    style.border = control_border(
                                        theme,
                                        status == text_editor::Status::Focused,
                                    );
                                    style
                                },
                            );
                            if self.job.is_none() {
                                editor.on_action(Message::Edit)
                            } else {
                                editor
                            }
                        },]
                        .spacing(8),
                    };
                    scrollable(
                        container(
                            column![
                                tabs,
                                text(status.clone()).wrapping(text::Wrapping::WordOrGlyph),
                                panel
                            ]
                            .spacing(12),
                        )
                        .width(Length::Fill)
                        .padding(12),
                    )
                    .into()
                }
            };
            pane_grid::Content::new(body)
        })
        .spacing(6)
        .on_resize(8, Message::Resize);
        let mut shell = column![menu_bar, project_tabs];
        if let Some(items) = menu_items {
            shell = shell.push(items);
        }
        if self.preferences.toolbar_visible {
            shell = shell.push(toolbar);
        }
        shell = shell.push(content).spacing(8).padding(8);
        container(shell)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::keyboard::on_key_press(|key, modifiers| Some(Message::KeyPressed(key, modifiers)))
    }
}

fn normalized_panel_ratio(candidate: f32, fallback_percent: u8) -> f32 {
    if candidate.is_finite() {
        candidate.clamp(0.2, 0.7)
    } else {
        f32::from(fallback_percent.clamp(20, 70)) / 100.0
    }
}

fn shortcut_key_labels() -> Vec<String> {
    ["Enter", "Escape", "[", "]"]
        .into_iter()
        .map(str::to_owned)
        .chain(('A'..='Z').map(|character| character.to_string()))
        .chain(('0'..='9').map(|character| character.to_string()))
        .collect()
}

fn apply_path_selection(
    display: &mut String,
    selected_path: &mut Option<PathBuf>,
    selected: Option<PathBuf>,
) {
    if let Some(path) = selected {
        *display = path.to_string_lossy().into_owned();
        *selected_path = Some(path);
    }
}

fn picker_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let mut style = pick_list::default(theme, status);
    style.border = control_border(theme, status != pick_list::Status::Active);
    style
}

fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mut style = text_input::default(theme, status);
    style.border = control_border(theme, status == text_input::Status::Focused);
    style.placeholder = theme.extended_palette().background.base.text;
    style
}

/// Open the native workbench on the main thread.
///
/// The application runs one handler at a time on a background thread. Each
/// handler is responsible for input limits, error handling, and checking for
/// cancellation. An empty operation list displays the shell without a Run action.
pub fn run(title: &'static str, ops: &'static [Operation], handler: Handler) -> iced::Result {
    run_workbench(title, ops, handler, None)
}

/// Open the workbench with appearance and workspace preferences saved under a stable application key.
///
/// Keys use lowercase ASCII letters, digits and hyphens. The application stores only
/// its theme, scale, toolbar visibility, pane ratio and shortcuts in the user
/// configuration directory. Input paths and drafts are session data. When
/// preferences are unavailable, the workbench uses defaults.
pub fn run_with_saved_appearance(
    title: &'static str,
    app_key: &'static str,
    ops: &'static [Operation],
    handler: Handler,
) -> iced::Result {
    run_workbench(title, ops, handler, app_preferences_path(app_key))
}

fn run_workbench(
    title: &'static str,
    ops: &'static [Operation],
    handler: Handler,
    preferences_path: Option<PathBuf>,
) -> iced::Result {
    let (preferences, preferences_error) = match preferences_path.as_deref().map(Preferences::load)
    {
        Some(Ok(preferences)) => (preferences, None),
        Some(Err(error)) => (
            Preferences::default(),
            Some(format!("Could not load appearance preferences: {error}")),
        ),
        None => (Preferences::default(), None),
    };
    let (mut panes, controls) = pane_grid::State::new(WorkspacePane::Controls);
    let (_, split) = panes
        .split(
            pane_grid::Axis::Vertical,
            controls,
            WorkspacePane::Workspace,
        )
        .expect("initial workspace split is valid");
    panes.resize(split, f32::from(preferences.panel_ratio) / 100.0);
    iced::application(title, State::update, State::view)
        .subscription(State::subscription)
        .theme(|s: &State| preset(&s.preferences.theme))
        .scale_factor(|s: &State| f64::from(s.preferences.scale) / 100.0)
        .run_with(move || {
            (
                State {
                    ops,
                    handler,
                    projects: vec![Project::new(1, ops)],
                    active_project: 1,
                    next_project_id: 2,
                    preferences,
                    preferences_path,
                    preferences_error,
                    job: None,
                    job_project: None,
                    status: "Ready".into(),
                    panes,
                    workspace_split: split,
                    active_menu: None,
                },
                Task::none(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_for_test() -> State {
        let ops = &[Operation {
            id: "test",
            label: "Test",
            hint: "Test operation",
        }];
        let (mut panes, controls) = pane_grid::State::new(WorkspacePane::Controls);
        let (_, workspace_split) = panes
            .split(
                pane_grid::Axis::Vertical,
                controls,
                WorkspacePane::Workspace,
            )
            .expect("test workspace split is valid");
        State {
            ops,
            handler: |_request, _job| Ok(Outcome::default()),
            projects: vec![Project::new(1, ops)],
            active_project: 1,
            next_project_id: 2,
            preferences: Preferences::default(),
            preferences_path: None,
            preferences_error: None,
            job: None,
            job_project: None,
            status: "Ready".into(),
            panes,
            workspace_split,
            active_menu: None,
        }
    }

    fn luminance(color: Color) -> f32 {
        let linear = |channel: f32| {
            if channel <= 0.04045 {
                channel / 12.92
            } else {
                ((channel + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }

    #[test]
    fn bounded_table_preview_parses_rows_and_preserves_trailing_warnings() {
        let report = concat!(
            r#"{"rows":1000,"headers":["id","name"],"preview":[["1","Ada"],["2","Lin"]]}"#,
            "\nWarning: original source was preserved."
        );
        let table = preview_table(report).unwrap();
        assert_eq!(table.headers, ["id", "name"]);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.total_rows, 1000);
        assert_eq!(table.warnings, "Warning: original source was preserved.");
        let _view = preview_table_view(table);
    }

    #[test]
    fn bounded_table_preview_rejects_unbounded_or_malformed_shapes() {
        let too_many_columns = serde_json::json!({
            "rows": 0,
            "headers": (0..=MAX_PREVIEW_TABLE_COLUMNS)
                .map(|index| format!("column-{index}"))
                .collect::<Vec<_>>(),
            "preview": []
        });
        assert!(preview_table(&too_many_columns.to_string()).is_none());
        let too_many_cells = serde_json::json!({
            "rows": 65,
            "headers": (0..128).map(|index| format!("column-{index}")).collect::<Vec<_>>(),
            "preview": vec![vec![""; 128]; 65]
        });
        assert!(preview_table(&too_many_cells.to_string()).is_none());
        assert!(
            preview_table(r#"{"rows":1,"headers":["a","b"],"preview":[["only one"]]}"#).is_none()
        );
        assert!(preview_table("ordinary textual result").is_none());
    }

    #[test]
    fn each_preset_has_readable_workbench_text() {
        for (name, ..) in THEMES {
            let theme = preset(name);
            let colors = theme.extended_palette();
            for pair in [
                colors.background.base,
                colors.background.weak,
                colors.primary.strong,
                colors.secondary.base,
                colors.secondary.strong,
            ] {
                let a = luminance(pair.color);
                let b = luminance(pair.text);
                let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                assert!(ratio >= 4.5, "{name} foreground contrast was {ratio:.2}:1");
            }
        }
    }

    #[test]
    fn cancelling_a_native_picker_preserves_the_typed_path() {
        let mut path = "manual/path.csv".to_owned();
        let mut selected = None;
        apply_path_selection(&mut path, &mut selected, None);
        assert_eq!(path, "manual/path.csv");
        assert_eq!(selected, None);
        apply_path_selection(
            &mut path,
            &mut selected,
            Some(PathBuf::from("/tmp/selected file.csv")),
        );
        assert_eq!(path, "/tmp/selected file.csv");
        assert_eq!(selected, Some(PathBuf::from("/tmp/selected file.csv")));
    }

    #[cfg(unix)]
    #[test]
    fn native_picker_keeps_non_unicode_filesystem_path() {
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(std::ffi::OsString::from_vec(b"/tmp/name-\xff.csv".to_vec()));
        let mut display = String::new();
        let mut selected = None;
        apply_path_selection(&mut display, &mut selected, Some(path.clone()));
        assert_eq!(selected, Some(path));
        assert!(display.contains('\u{fffd}'));
    }

    #[test]
    fn workspace_split_ratio_stays_within_safe_bounds() {
        assert_eq!(normalized_panel_ratio(0.1, 34), 0.2);
        assert_eq!(normalized_panel_ratio(0.9, 34), 0.7);
        assert_eq!(normalized_panel_ratio(f32::NAN, 44), 0.44);
    }

    #[test]
    fn keyboard_panel_resize_updates_the_persistent_ratio_with_bounds() {
        let mut state = state_for_test();
        state.resize_controls(-5);
        assert_eq!(state.preferences.panel_ratio, 29);
        state.preferences.panel_ratio = 20;
        state.resize_controls(-5);
        assert_eq!(state.preferences.panel_ratio, 20);
        state.preferences.panel_ratio = 70;
        state.resize_controls(5);
        assert_eq!(state.preferences.panel_ratio, 70);
        state.resize_controls(5);
        assert_eq!(state.preferences.panel_ratio, 70);
    }

    #[test]
    fn toolbar_visibility_command_toggles_the_persisted_preference() {
        let mut state = state_for_test();
        assert!(state.preferences.toolbar_visible);
        let _ = state.update(Message::Command(WorkbenchCommand::ToggleToolbar));
        assert!(!state.preferences.toolbar_visible);
        let _ = state.update(Message::Command(WorkbenchCommand::ToggleToolbar));
        assert!(state.preferences.toolbar_visible);
    }

    #[test]
    fn project_tabs_keep_inputs_drafts_and_results_separate() {
        let mut state = state_for_test();
        state.active_project_mut().input = "first.csv".into();
        state.active_project_mut().parameter = "first parameter".into();
        state.active_project_mut().preview = "first result".into();
        state.active_project_mut().editor = text_editor::Content::with_text("first draft");

        state.add_project();
        let second = state.active_project;
        assert_eq!(second, 2);
        assert!(state.active_project().input.is_empty());
        assert!(state.active_project().preview.is_empty());
        state.active_project_mut().input = "second.csv".into();
        state.active_project_mut().preview = "second result".into();

        state.active_project = 1;
        assert_eq!(state.active_project().input, "first.csv");
        assert_eq!(state.active_project().parameter, "first parameter");
        assert_eq!(state.active_project().preview, "first result");
        assert_eq!(state.active_project().editor.text(), "first draft\n");
        state.active_project = second;
        assert_eq!(state.active_project().input, "second.csv");
        assert_eq!(state.active_project().preview, "second result");
    }

    #[test]
    fn project_tabs_cycle_and_close_without_leaving_a_stale_active_id() {
        let mut state = state_for_test();
        state.add_project();
        state.add_project();
        assert_eq!(state.active_project, 3);

        state.active_project = 1;
        let _ = state.update(Message::KeyPressed(Key::Named(Named::Tab), Modifiers::CTRL));
        assert_eq!(state.active_project, 2);
        let _ = state.update(Message::KeyPressed(
            Key::Named(Named::Tab),
            Modifiers::CTRL | Modifiers::SHIFT,
        ));
        assert_eq!(state.active_project, 1);

        state.active_project = 3;
        state.cycle_project(false);
        assert_eq!(state.active_project, 1);
        state.cycle_project(true);
        assert_eq!(state.active_project, 3);

        state.close_project(3);
        assert_eq!(state.active_project, 2);
        assert_eq!(state.projects.len(), 2);
        state.close_project(2);
        assert_eq!(state.active_project, 1);
        state.close_project(1);
        assert_eq!(state.projects.len(), 1);
    }

    #[test]
    fn project_count_is_bounded_and_delayed_picker_results_keep_their_owner() {
        let mut state = state_for_test();
        for _ in 1..MAX_PROJECT_TABS {
            state.add_project();
        }
        assert_eq!(state.projects.len(), MAX_PROJECT_TABS);
        let final_id = state.active_project;
        state.add_project();
        assert_eq!(state.projects.len(), MAX_PROJECT_TABS);
        assert_eq!(state.active_project, final_id);

        let original_id = state.projects[0].id;
        state.active_project = final_id;
        let _ = state.update(Message::InputPicked(
            original_id,
            Some(PathBuf::from("/tmp/original.csv")),
        ));
        assert_eq!(state.projects[0].input, "/tmp/original.csv");
        assert!(state.active_project().input.is_empty());
    }

    #[test]
    fn closing_a_project_requests_cancellation_for_its_running_job() {
        let mut state = state_for_test();
        state.add_project();
        let job = Job::new();
        state.job = Some(job.clone());
        state.job_project = Some(state.active_project);
        let closing = state.active_project;
        state.close_project(closing);
        assert_eq!(state.active_project, 1);
        assert!(state.job.is_some());
        assert!(job.check().is_err());
    }
}
