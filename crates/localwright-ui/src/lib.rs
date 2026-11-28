#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
mod commands;
mod preferences;
mod project_tabs;
mod session;
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
use project_tabs::{PersistedProjectTab, PersistedProjectTabs, MAX_PROJECT_ID, MAX_PROJECT_TABS};
pub use session::{
    WorkspaceLayout, WorkspaceProject, WorkspaceSession, WorkspaceView, MAX_SESSION_PROJECTS,
};
pub use shortcuts::{ShortcutBinding, ShortcutCommand, ShortcutKey, ShortcutRegistry};
use std::{fmt, path::PathBuf};
pub use table::{
    paginated_table, CellValue, SortDirection, TableAction, TableError, TableModel, TableState,
    MAX_TABLE_COLUMNS, MAX_TABLE_PAGE_SIZE,
};

/// One inspected column shown with a table workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableColumnInfo {
    /// Source column name.
    pub name: String,
    /// Logical type label supplied by the application.
    pub data_type: String,
    /// Whether this column contains missing values.
    pub nullable: bool,
    /// Number of missing values in the loaded table.
    pub missing: usize,
    /// Optional application-defined display label.
    pub label: Option<String>,
    /// Optional application-defined unit.
    pub unit: Option<String>,
}

/// Reusable native tabular workspace content supplied by an application.
#[derive(Clone, Debug, PartialEq)]
pub struct TableWorkspace {
    /// Workspace heading.
    pub title: String,
    /// Short description, such as row count and source file name.
    pub summary: String,
    /// Per-column type and metadata inspection.
    pub schema: Vec<TableColumnInfo>,
    /// Typed values rendered by the shared paginated table.
    pub table: TableModel,
}

/// Converts an application result payload into a native table workspace.
/// Return `Ok(None)` for results that are not table-workspace payloads.
pub type TableWorkspaceProvider = fn(&str) -> std::result::Result<Option<TableWorkspace>, String>;

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
    table_workspace_provider: Option<TableWorkspaceProvider>,
}

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
    table_workspace: Option<TableWorkspace>,
    table_filter: String,
    table_state: TableState,
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
            table_workspace: None,
            table_filter: String::new(),
            table_state: TableState::new(MAX_TABLE_PAGE_SIZE),
        }
    }

    fn from_saved(tab: &PersistedProjectTab, ops: &[Operation]) -> Self {
        let mut project = Self::new(tab.id, ops);
        project.title.clone_from(&tab.title);
        project
    }

    fn from_session(saved: &WorkspaceProject, id: u64) -> Self {
        Self {
            id,
            title: saved.title.clone(),
            selected: saved.operation.clone(),
            input: saved.input.clone(),
            input_selection: (!saved.input.is_empty()).then(|| PathBuf::from(&saved.input)),
            output: saved.output.clone(),
            output_selection: (!saved.output.is_empty()).then(|| PathBuf::from(&saved.output)),
            parameter: saved.parameter.clone(),
            editor: text_editor::Content::with_text(&saved.draft),
            preview: String::new(),
            workspace_tab: match saved.view {
                WorkspaceView::Results => WorkspaceTab::Result,
                WorkspaceView::Draft => WorkspaceTab::Draft,
            },
            table_workspace: None,
            table_filter: String::new(),
            table_state: TableState::new(MAX_TABLE_PAGE_SIZE),
        }
    }
}

fn restore_projects(saved: &PersistedProjectTabs, ops: &[Operation]) -> (Vec<Project>, u64, u64) {
    if saved.tabs.is_empty() {
        return (vec![Project::new(1, ops)], 1, 2);
    }
    let projects = saved
        .tabs
        .iter()
        .map(|tab| Project::from_saved(tab, ops))
        .collect::<Vec<_>>();
    let active = saved.active_id.unwrap_or(projects[0].id);
    (projects, active, saved.next_id())
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
    Table,
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
    SaveSession,
    SaveSessionPicked(Option<PathBuf>),
    RestoreSession,
    RestoreSessionPicked(Option<PathBuf>),
    SessionRestored(std::result::Result<WorkspaceSession, String>),
    Run,
    Cancel,
    KeyPressed(Key, Modifiers),
    Done(u64, Result<Outcome>),
    Resize(pane_grid::ResizeEvent),
    WorkspaceTab(WorkspaceTab),
    TableFilter(String),
    TableAction(TableAction),
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
        if id > MAX_PROJECT_ID {
            self.status = "Project identifier space exhausted; restart the workbench".into();
            return;
        }
        let Some(next_id) = id.checked_add(1) else {
            self.status = "Project identifier space exhausted; restart the workbench".into();
            return;
        };
        self.next_project_id = next_id;
        self.projects.push(Project::new(id, self.ops));
        self.active_project = id;
        self.announce_project_selection(format!("Opened {}", self.active_project().title));
        self.persist_project_tabs();
    }

    fn close_project(&mut self, id: u64) {
        if self.projects.len() <= 1 {
            self.status =
                "Keep at least one project tab open. The current tab remains selected.".into();
            return;
        }
        let Some(index) = self.projects.iter().position(|project| project.id == id) else {
            return;
        };
        let closed_title = self.projects[index].title.clone();
        if self.job_project == Some(id) {
            if let Some(job) = &self.job {
                job.cancel();
            }
        }
        self.projects.remove(index);
        if self.active_project == id {
            self.active_project = self.projects[index.min(self.projects.len() - 1)].id;
        }
        let active_title = self.active_project().title.clone();
        self.announce_project_selection(format!("Closed {closed_title}; selected {active_title}"));
        if self.job_project == Some(id) {
            self.status
                .push_str("; waiting for the cancelled operation to stop");
        }
        self.persist_project_tabs();
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
        self.activate_project(self.projects[next].id);
    }

    fn activate_project(&mut self, id: u64) {
        if self.active_project == id {
            return;
        }
        if !self.projects.iter().any(|project| project.id == id) {
            return;
        }
        self.active_project = id;
        let title = self.active_project().title.clone();
        self.announce_project_selection(format!("Selected {title}"));
        self.persist_project_tabs();
    }

    fn announce_project_selection(&mut self, action: String) {
        let index = self
            .projects
            .iter()
            .position(|project| project.id == self.active_project)
            .unwrap_or(0);
        self.status = format!(
            "{action}. Current project tab {} of {}: {}",
            index + 1,
            self.projects.len(),
            self.active_project().title
        );
    }

    fn persist_project_tabs(&mut self) {
        self.preferences.project_tabs = PersistedProjectTabs {
            tabs: self
                .projects
                .iter()
                .map(|project| PersistedProjectTab {
                    id: project.id,
                    title: project.title.clone(),
                })
                .collect(),
            active_id: Some(self.active_project),
        };
        self.save_preferences();
    }

    fn workspace_session(&self) -> Result<WorkspaceSession> {
        let projects = self
            .projects
            .iter()
            .map(|project| {
                let selected_path = |display: &str, path: &Option<PathBuf>| -> Result<String> {
                    match path {
                        Some(path) => path
                            .to_str()
                            .map(str::to_owned)
                            .ok_or_else(|| Error::input("Session paths must be valid UTF-8")),
                        None => Ok(display.to_owned()),
                    }
                };
                if !project.selected.is_empty()
                    && !self
                        .ops
                        .iter()
                        .any(|operation| operation.id == project.selected)
                {
                    return Err(Error::input(
                        "The active operation is not registered by this application",
                    ));
                }
                Ok(WorkspaceProject {
                    title: project.title.clone(),
                    operation: project.selected.clone(),
                    input: selected_path(&project.input, &project.input_selection)?,
                    output: selected_path(&project.output, &project.output_selection)?,
                    parameter: project.parameter.clone(),
                    draft: project.editor.text(),
                    view: match project.workspace_tab {
                        WorkspaceTab::Draft => WorkspaceView::Draft,
                        WorkspaceTab::Result | WorkspaceTab::Table => WorkspaceView::Results,
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let active_project = self
            .projects
            .iter()
            .position(|project| project.id == self.active_project)
            .ok_or_else(|| Error::input("The active workbench project is unavailable"))?;
        let session = WorkspaceSession {
            schema_version: 1,
            layout: WorkspaceLayout {
                controls_ratio: self.preferences.panel_ratio,
                toolbar_visible: self.preferences.toolbar_visible,
            },
            active_project,
            projects,
        };
        session.validate()?;
        Ok(session)
    }

    fn restore_workspace_session(
        &mut self,
        session: WorkspaceSession,
    ) -> std::result::Result<usize, String> {
        session.validate().map_err(|error| error.to_string())?;
        if self.projects.len().saturating_add(session.projects.len()) > MAX_PROJECT_TABS {
            return Err(format!(
                "Restoring these tabs would exceed the {MAX_PROJECT_TABS}-tab workbench limit"
            ));
        }
        for project in &session.projects {
            if !project.operation.is_empty()
                && !self
                    .ops
                    .iter()
                    .any(|operation| operation.id == project.operation)
            {
                return Err(format!(
                    "Operation {:?} is not available in this application",
                    project.operation
                ));
            }
        }
        let count = session.projects.len();
        let first_id = self.next_project_id;
        let final_id = first_id
            .checked_add(count as u64)
            .ok_or_else(|| "Project identifier space exhausted".to_owned())?;
        if final_id.saturating_sub(1) > MAX_PROJECT_ID {
            return Err("Project identifier space exhausted".into());
        }
        let restored = session
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| Project::from_session(project, first_id + index as u64))
            .collect::<Vec<_>>();
        let active_id = restored[session.active_project].id;
        self.projects.extend(restored);
        self.active_project = active_id;
        self.next_project_id = final_id;
        self.preferences.panel_ratio = session.layout.controls_ratio;
        self.preferences.toolbar_visible = session.layout.toolbar_visible;
        self.panes.resize(
            self.workspace_split,
            f32::from(self.preferences.panel_ratio) / 100.0,
        );
        self.persist_project_tabs();
        self.announce_project_selection(format!("Restored {} project tabs", count));
        Ok(count)
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
            Message::ActivateProject(id) => self.activate_project(id),
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
            Message::SaveSession => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Save workspace session")
                            .add_filter("Localwright workspace session (.json)", &["json"])
                            .save_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    Message::SaveSessionPicked,
                );
            }
            Message::SaveSessionPicked(Some(path)) => {
                let result = self
                    .workspace_session()
                    .and_then(|session| session.save(&path))
                    .map_err(|error| error.to_string());
                self.status = match result {
                    Ok(()) => format!("Saved workspace session to {}", path.display()),
                    Err(error) => format!("Could not save workspace session: {error}"),
                };
            }
            Message::SaveSessionPicked(None) => {}
            Message::RestoreSession => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Restore workspace session")
                            .add_filter("JSON session files", &["json"])
                            .pick_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    Message::RestoreSessionPicked,
                );
            }
            Message::RestoreSessionPicked(Some(path)) => {
                return Task::perform(
                    async move { WorkspaceSession::load(&path).map_err(|error| error.to_string()) },
                    Message::SessionRestored,
                );
            }
            Message::RestoreSessionPicked(None) => {}
            Message::SessionRestored(Ok(session)) => {
                match self.restore_workspace_session(session) {
                    Ok(restored) => {
                        self.status = format!(
                            "Restored {restored} project tabs. No operation was run; review paths before choosing Run."
                        );
                    }
                    Err(error) => self.status = format!("Could not restore session: {error}"),
                }
            }
            Message::SessionRestored(Err(error)) => {
                self.status = format!("Could not read workspace session: {error}");
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
                    Some(ShortcutCommand::NewProject) => self.add_project(),
                    Some(ShortcutCommand::CloseProject) => self.close_project(self.active_project),
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
                        let table_workspace = self
                            .table_workspace_provider
                            .map(|provider| provider(&o.preview))
                            .unwrap_or(Ok(None));
                        match table_workspace {
                            Ok(workspace) => {
                                project.table_state = TableState::new(MAX_TABLE_PAGE_SIZE);
                                project.table_filter.clear();
                                project.table_workspace = workspace;
                                project.preview = project
                                    .table_workspace
                                    .as_ref()
                                    .map(|table| format!("{}\n{}", table.title, table.summary))
                                    .unwrap_or(o.preview);
                            }
                            Err(error) => {
                                project.table_workspace = None;
                                project.preview =
                                    format!("Could not prepare the table workspace: {error}");
                                self.status = "Table workspace unavailable".into();
                            }
                        }
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
            Message::TableFilter(filter) => {
                let status = {
                    let project = self.active_project_mut();
                    if let Some(workspace) = &mut project.table_workspace {
                        match workspace.table.filter_contains(&filter) {
                            Ok(()) => {
                                project.table_filter = filter;
                                project.table_state.set_page(0, workspace.table.row_count());
                                project
                                    .table_state
                                    .select_row(usize::MAX, workspace.table.row_count());
                                format!(
                                    "Showing {} matching loaded rows",
                                    workspace.table.row_count()
                                )
                            }
                            Err(error) => error.to_string(),
                        }
                    } else {
                        String::new()
                    }
                };
                if !status.is_empty() {
                    self.status = status;
                }
            }
            Message::TableAction(action) => {
                let project = self.active_project_mut();
                if let Some(workspace) = &mut project.table_workspace {
                    let row_count = workspace.table.row_count();
                    match action {
                        TableAction::PageChanged(page) => {
                            project.table_state.set_page(page, row_count)
                        }
                        TableAction::SelectRow(row) => {
                            project.table_state.select_row(row, row_count)
                        }
                        TableAction::SortBy { column, direction } => {
                            if workspace.table.sort_by(column, direction).is_ok() {
                                project.table_state.set_sort(column, direction);
                            }
                        }
                    }
                }
            }
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
                    WorkbenchCommand::SaveSession => self.update(Message::SaveSession),
                    WorkbenchCommand::RestoreSession => self.update(Message::RestoreSession),
                    WorkbenchCommand::NewProject => self.update(Message::NewProject),
                    WorkbenchCommand::CloseProject => {
                        self.update(Message::CloseProject(self.active_project))
                    }
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
        for (index, tab) in self.projects.iter().enumerate() {
            let active = tab.id == self.active_project;
            let label = if active {
                format!(
                    "{} (selected, {} of {})",
                    tab.title,
                    index + 1,
                    self.projects.len()
                )
            } else {
                format!("{} ({} of {})", tab.title, index + 1, self.projects.len())
            };
            project_tabs =
                project_tabs.push(button(text(label)).on_press(Message::ActivateProject(tab.id)));
            let close = button(text(format!("Close project tab {}", tab.title)));
            project_tabs = project_tabs.push(if self.projects.len() > 1 {
                close.on_press(Message::CloseProject(tab.id))
            } else {
                close
            });
        }
        let add_project = button("New project tab (Ctrl/Command+T)");
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
                                text("New project shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::NewProject)
                                        .map(|binding| binding.key.label()),
                                    |label| Message::ShortcutKey(
                                        ShortcutCommand::NewProject,
                                        label
                                    )
                                )
                                .style(picker_style),
                                text("Close project shortcut"),
                                pick_list(
                                    shortcut_keys.clone(),
                                    self.preferences
                                        .shortcuts
                                        .binding(ShortcutCommand::CloseProject)
                                        .map(|binding| binding.key.label()),
                                    |label| Message::ShortcutKey(
                                        ShortcutCommand::CloseProject,
                                        label
                                    )
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
                    let mut tabs = row![
                        button("Results").on_press(Message::WorkspaceTab(WorkspaceTab::Result)),
                        button("Draft / transcript")
                            .on_press(Message::WorkspaceTab(WorkspaceTab::Draft)),
                    ]
                    .spacing(8);
                    if self.table_workspace_provider.is_some() {
                        tabs = tabs.push(
                            button("Data table")
                                .on_press(Message::WorkspaceTab(WorkspaceTab::Table)),
                        );
                    }
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
                        WorkspaceTab::Table => match &project.table_workspace {
                            Some(workspace) => {
                                let schema = column(workspace.schema.iter().map(|column| {
                                    let mut details = format!(
                                        "{} · {} · {} missing",
                                        column.name, column.data_type, column.missing
                                    );
                                    if let Some(label) = &column.label {
                                        details.push_str(&format!(" · {label}"));
                                    }
                                    if let Some(unit) = &column.unit {
                                        details.push_str(&format!(" ({unit})"));
                                    }
                                    text(details).into()
                                }))
                                .spacing(2);
                                column![
                                    text(&workspace.title),
                                    text(&workspace.summary),
                                    text("Filter loaded preview rows"),
                                    text_input("Search values in the loaded table", &project.table_filter)
                                        .on_input(Message::TableFilter)
                                        .padding(8),
                                    text(format!("{} rows match the filter", workspace.table.row_count())),
                                    text("Schema and column metadata"),
                                    scrollable(schema).height(Length::Fixed(120.0)),
                                    paginated_table(
                                        &workspace.table,
                                        &project.table_state,
                                        Message::TableAction
                                    ),
                                ]
                                .spacing(8)
                            }
                            None => column![
                                text("No typed table is available for this result."),
                                text("Load a supported table and run the Inspect typed table operation."),
                            ]
                            .spacing(8),
                        },
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
    run_workbench(title, ops, handler, None, None)
}

/// Open the workbench with appearance and workspace preferences saved under a stable application key.
///
/// Keys use lowercase ASCII letters, digits and hyphens. The application stores
/// theme, scale, toolbar visibility, pane ratio, shortcuts and bounded project
/// tab titles/order/selection in the user configuration directory. Project paths,
/// drafts and results are session data. When
/// preferences are unavailable, the workbench uses defaults.
pub fn run_with_saved_appearance(
    title: &'static str,
    app_key: &'static str,
    ops: &'static [Operation],
    handler: Handler,
) -> iced::Result {
    run_workbench(title, ops, handler, app_preferences_path(app_key), None)
}

/// Open the standard workbench with a reusable paginated table workspace.
///
/// The provider receives the operation result payload after the background
/// handler completes. It should return `Ok(None)` for ordinary results and a
/// bounded [`TableWorkspace`] for table payloads. The Results and Draft tabs
/// remain available; the extra Table tab owns paging, selection and sort state
/// independently for each project.
pub fn run_with_saved_appearance_and_table_workspace(
    title: &'static str,
    app_key: &'static str,
    ops: &'static [Operation],
    handler: Handler,
    provider: TableWorkspaceProvider,
) -> iced::Result {
    run_workbench(
        title,
        ops,
        handler,
        app_preferences_path(app_key),
        Some(provider),
    )
}

fn run_workbench(
    title: &'static str,
    ops: &'static [Operation],
    handler: Handler,
    preferences_path: Option<PathBuf>,
    table_workspace_provider: Option<TableWorkspaceProvider>,
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
            let (projects, active_project, next_project_id) =
                restore_projects(&preferences.project_tabs, ops);
            let active_index = projects
                .iter()
                .position(|project| project.id == active_project)
                .unwrap_or(0);
            let status = format!(
                "Ready. Current project tab {} of {}: {}",
                active_index + 1,
                projects.len(),
                projects[active_index].title
            );
            (
                State {
                    ops,
                    handler,
                    projects,
                    active_project,
                    next_project_id,
                    preferences,
                    preferences_path,
                    preferences_error,
                    job: None,
                    job_project: None,
                    status,
                    panes,
                    workspace_split: split,
                    active_menu: None,
                    table_workspace_provider,
                },
                Task::none(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_table_workspace(
        _payload: &str,
    ) -> std::result::Result<Option<TableWorkspace>, String> {
        let rows = (0..205)
            .map(|value| {
                vec![
                    CellValue::Integer((204 - value) as i64),
                    match value {
                        0 => CellValue::Text(String::new()),
                        1 => CellValue::Null,
                        _ => CellValue::Text(format!("row {value}")),
                    },
                ]
            })
            .collect();
        let table = TableModel::new(vec!["score".into(), "note".into()], rows)
            .map_err(|error| error.to_string())?;
        Ok(Some(TableWorkspace {
            title: "Typed data".into(),
            summary: "205 rows".into(),
            schema: vec![TableColumnInfo {
                name: "score".into(),
                data_type: "int64".into(),
                nullable: true,
                missing: 1,
                label: None,
                unit: None,
            }],
            table,
        }))
    }

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
            table_workspace_provider: None,
        }
    }

    #[test]
    fn table_workspace_provider_renders_data_and_updates_page_sort_selection() {
        let mut state = state_for_test();
        state.table_workspace_provider = Some(sample_table_workspace);
        let _ = state.update(Message::Done(
            1,
            Ok(Outcome {
                preview: "table payload".into(),
                ..Outcome::default()
            }),
        ));
        let project = state.active_project();
        let workspace = project.table_workspace.as_ref().unwrap();
        assert_eq!(workspace.table.row_count(), 205);
        assert_eq!(workspace.schema[0].data_type, "int64");
        assert!(matches!(
            workspace.table.value_at(0, 1),
            Some(CellValue::Text(value)) if value.is_empty()
        ));
        assert_eq!(workspace.table.value_at(1, 1), Some(&CellValue::Null));
        assert!(project.preview.contains("205 rows"));

        let _ = state.update(Message::TableAction(TableAction::PageChanged(99)));
        assert_eq!(state.active_project().table_state.current_page(205), 2);
        let _ = state.update(Message::TableAction(TableAction::SelectRow(102)));
        assert_eq!(state.active_project().table_state.selected_row(), Some(102));
        let _ = state.update(Message::TableAction(TableAction::SortBy {
            column: 0,
            direction: SortDirection::Ascending,
        }));
        assert_eq!(
            state.active_project().table_state.sort(),
            Some((0, SortDirection::Ascending))
        );
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

        state.activate_project(1);
        assert_eq!(state.active_project().input, "first.csv");
        assert_eq!(state.active_project().parameter, "first parameter");
        assert_eq!(state.active_project().preview, "first result");
        assert_eq!(state.active_project().editor.text(), "first draft\n");
        state.activate_project(second);
        assert_eq!(state.active_project().input, "second.csv");
        assert_eq!(state.active_project().preview, "second result");
    }

    #[test]
    fn project_tabs_cycle_and_close_without_leaving_a_stale_active_id() {
        let mut state = state_for_test();
        state.add_project();
        state.add_project();
        assert_eq!(state.active_project, 3);

        state.activate_project(1);
        let _ = state.update(Message::KeyPressed(Key::Named(Named::Tab), Modifiers::CTRL));
        assert_eq!(state.active_project, 2);
        let _ = state.update(Message::KeyPressed(
            Key::Named(Named::Tab),
            Modifiers::CTRL | Modifiers::SHIFT,
        ));
        assert_eq!(state.active_project, 1);

        state.activate_project(3);
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
        state.activate_project(final_id);
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

    #[test]
    fn project_tabs_restore_order_selection_and_next_identifier_without_session_data() {
        let ops = &[Operation {
            id: "test",
            label: "Test",
            hint: "Test operation",
        }];
        let saved = PersistedProjectTabs {
            tabs: vec![
                PersistedProjectTab {
                    id: 3,
                    title: "Inputs".into(),
                },
                PersistedProjectTab {
                    id: 8,
                    title: "Review".into(),
                },
            ],
            active_id: Some(8),
        };
        let (projects, active, next) = restore_projects(&saved, ops);
        assert_eq!(
            projects
                .iter()
                .map(|project| project.id)
                .collect::<Vec<_>>(),
            vec![3, 8]
        );
        assert_eq!(projects[0].title, "Inputs");
        assert_eq!(projects[1].title, "Review");
        assert_eq!(active, 8);
        assert_eq!(next, 9);
        assert!(projects.iter().all(|project| {
            project.input.is_empty()
                && project.output.is_empty()
                && project.parameter.is_empty()
                && project.preview.is_empty()
        }));
    }

    #[test]
    fn workspace_session_restores_operation_context_by_appending_without_running_it() {
        let mut source = state_for_test();
        source.active_project_mut().input = "/tmp/first.csv".into();
        source.active_project_mut().parameter = "first parameter".into();
        source.active_project_mut().editor = text_editor::Content::with_text("saved draft");
        source.active_project_mut().workspace_tab = WorkspaceTab::Draft;
        source.preferences.panel_ratio = 48;
        source.preferences.toolbar_visible = false;
        source.add_project();
        source.active_project_mut().output = "/tmp/output.csv".into();
        source.active_project_mut().parameter = "second parameter".into();

        let snapshot = source.workspace_session().unwrap();
        assert_eq!(snapshot.active_project, 1);
        assert_eq!(snapshot.projects.len(), 2);

        let mut restored = state_for_test();
        assert_eq!(restored.restore_workspace_session(snapshot).unwrap(), 2);
        assert_eq!(restored.projects.len(), 3);
        assert_eq!(restored.active_project, 3);
        assert_eq!(restored.next_project_id, 4);
        assert_eq!(restored.preferences.panel_ratio, 48);
        assert!(!restored.preferences.toolbar_visible);
        assert!(restored.job.is_none());
        let active = restored.active_project();
        assert_eq!(active.selected, "test");
        assert_eq!(active.output, "/tmp/output.csv");
        assert_eq!(active.parameter, "second parameter");
        assert!(active.preview.is_empty());
        assert!(matches!(active.workspace_tab, WorkspaceTab::Result));

        restored.activate_project(2);
        assert_eq!(restored.active_project().input, "/tmp/first.csv");
        assert_eq!(restored.active_project().editor.text(), "saved draft\n");
        assert!(matches!(
            restored.active_project().workspace_tab,
            WorkspaceTab::Draft
        ));
    }

    #[test]
    fn session_restore_rejects_unknown_app_operations_without_changing_workspace() {
        let mut state = state_for_test();
        let mut session = state.workspace_session().unwrap();
        session.projects[0].operation = "not-registered".into();
        assert!(state.restore_workspace_session(session).is_err());
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.active_project, 1);
        assert!(state.job.is_none());
    }

    #[test]
    fn project_tab_shortcuts_open_close_and_persist_selection_metadata() {
        let mut state = state_for_test();
        #[cfg(target_os = "macos")]
        let command = Modifiers::LOGO;
        #[cfg(not(target_os = "macos"))]
        let command = Modifiers::CTRL;
        let _ = state.update(Message::KeyPressed(Key::Character("t".into()), command));
        assert_eq!(state.projects.len(), 2);
        let opened = state.active_project;
        assert_eq!(state.preferences.project_tabs.active_id, Some(opened));
        assert_eq!(state.preferences.project_tabs.tabs.len(), 2);

        let _ = state.update(Message::KeyPressed(Key::Character("w".into()), command));
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.active_project, 1);
        assert_eq!(state.preferences.project_tabs.active_id, Some(1));
        assert!(state.status.contains("Current project tab 1 of 1"));
    }

    #[test]
    fn adding_a_project_stops_before_identifier_headroom_is_exhausted() {
        let mut state = state_for_test();
        state.projects[0].id = MAX_PROJECT_ID;
        state.active_project = MAX_PROJECT_ID;
        state.next_project_id = MAX_PROJECT_ID + 1;

        state.add_project();

        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.active_project, MAX_PROJECT_ID);
        assert_eq!(state.next_project_id, MAX_PROJECT_ID + 1);
        assert!(state.status.contains("identifier space exhausted"));
    }
}
