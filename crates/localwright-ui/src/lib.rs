#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
mod preferences;
mod shortcuts;
use iced::{
    keyboard::{Key, Modifiers},
    theme::Palette,
    widget::{
        button, column, container, pane_grid, pick_list, row, scrollable, text, text_editor,
        text_input,
    },
    Border, Color, Element, Length, Subscription, Task, Theme,
};
use localwright_core::{Error, ErrorKind, Handler, Job, Operation, Outcome, Request, Result};
use preferences::{app_preferences_path, Preferences};
pub use shortcuts::{ShortcutBinding, ShortcutCommand, ShortcutKey, ShortcutRegistry};
use std::{fmt, path::PathBuf};

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
    selected: String,
    input: String,
    input_selection: Option<PathBuf>,
    output: String,
    output_selection: Option<PathBuf>,
    parameter: String,
    editor: text_editor::Content,
    preview: String,
    preferences: Preferences,
    preferences_path: Option<PathBuf>,
    preferences_error: Option<String>,
    job: Option<Job>,
    status: String,
    panes: pane_grid::State<WorkspacePane>,
    workspace_tab: WorkspaceTab,
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
    Select(&'static str),
    Input(String),
    Output(String),
    Parameter(String),
    Edit(text_editor::Action),
    Theme(String),
    Scale(u16),
    ShortcutKey(ShortcutCommand, String),
    PickInput,
    InputPicked(Option<PathBuf>),
    PickOutput,
    OutputPicked(Option<PathBuf>),
    Run,
    Cancel,
    KeyPressed(Key, Modifiers),
    Done(Result<Outcome>),
    Resize(pane_grid::ResizeEvent),
    WorkspaceTab(WorkspaceTab),
}
impl State {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Select(v) => self.selected = v.into(),
            Message::Input(v) => {
                self.input = v;
                self.input_selection = None;
            }
            Message::Output(v) => {
                self.output = v;
                self.output_selection = None;
            }
            Message::Parameter(v) => self.parameter = v,
            Message::Edit(a) => self.editor.perform(a),
            Message::Theme(v) => {
                self.preferences.theme = v;
                self.save_preferences();
            }
            Message::Scale(v) => {
                self.preferences.scale = v;
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
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Select input file")
                            .pick_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    Message::InputPicked,
                );
            }
            Message::InputPicked(path) => {
                apply_path_selection(&mut self.input, &mut self.input_selection, path);
            }
            Message::PickOutput => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("Choose output file")
                            .save_file()
                            .await
                            .map(|file| file.path().to_path_buf())
                    },
                    Message::OutputPicked,
                );
            }
            Message::OutputPicked(path) => {
                apply_path_selection(&mut self.output, &mut self.output_selection, path);
            }
            Message::Cancel => {
                if let Some(j) = &self.job {
                    j.cancel();
                    self.status = "Cancellation requested; waiting for operation".into();
                }
            }
            Message::KeyPressed(key, modifiers) => {
                match self.preferences.shortcuts.resolve(&key, modifiers) {
                    Some(ShortcutCommand::Run) => return self.update(Message::Run),
                    Some(ShortcutCommand::Cancel) => return self.update(Message::Cancel),
                    None => {}
                }
            }
            Message::Run => {
                if self.job.is_some() {
                    return Task::none();
                }
                let job = Job::new();
                self.job = Some(job.clone());
                self.status = "Running operation".into();
                self.preview.clear();
                let r = Request {
                    operation: self.selected.clone(),
                    input: self
                        .input_selection
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&self.input)),
                    output: self
                        .output_selection
                        .clone()
                        .unwrap_or_else(|| PathBuf::from(&self.output)),
                    parameter: self.parameter.clone(),
                    draft: self.editor.text(),
                };
                let handler = self.handler;
                let (tx, rx) = futures::channel::oneshot::channel();
                let spawn = std::thread::Builder::new()
                    .name("localwright-job".into())
                    .spawn(move || {
                        let _ = tx.send(handler(r, job));
                    });
                if let Err(e) = spawn {
                    self.job = None;
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
                    Message::Done,
                );
            }
            Message::Done(result) => {
                self.job = None;
                match result {
                    Ok(o) => {
                        self.status = if let Some(p) = o.written {
                            format!("Wrote {}", p.display())
                        } else {
                            "Finished".into()
                        };
                        self.preview = o.preview;
                        for warning in o.warnings {
                            self.preview.push_str(&format!("\nWarning: {warning}"));
                        }
                        if let Some(d) = o.draft {
                            self.editor = text_editor::Content::with_text(&d);
                        }
                    }
                    Err(e) => {
                        self.status = format!("Error: {:?}", e.kind);
                        self.preview = e.message;
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
            Message::WorkspaceTab(tab) => self.workspace_tab = tab,
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
        let status = match &self.preferences_error {
            Some(error) => format!("{}\n{error}", self.status),
            None => self.status.clone(),
        };
        let ops = self.ops.iter().map(OperationChoice).collect::<Vec<_>>();
        let themes = THEMES.iter().map(|t| t.0.to_owned()).collect::<Vec<_>>();
        let shortcut_keys = shortcut_key_labels();
        let selected = ops.iter().copied().find(|o| o.0.id == self.selected);
        let hint = selected.map(|o| o.0.hint).unwrap_or("");
        let mut run = button("Run operation")
            .padding([8, 12])
            .style(|theme, status| {
                let mut style = button::primary(theme, status);
                let pair = theme.extended_palette().primary.strong;
                if status == button::Status::Hovered {
                    style.background = Some(pair.color.into());
                    style.border.width = 2.0;
                    style.border.color = pair.text;
                }
                style.border.radius = control_border(theme, false).radius;
                style
            });
        if self.job.is_none() && selected.is_some() {
            run = run.on_press(Message::Run);
        }
        let mut cancel = button("Cancel").padding([8, 12]).style(|theme, status| {
            let mut style = button::secondary(theme, status);
            if status == button::Status::Hovered {
                let pair = theme.extended_palette().secondary.strong;
                style.text_color = pair.text;
            }
            style.border.radius = control_border(theme, false).radius;
            style
        });
        if self.job.is_some() {
            cancel = cancel.on_press(Message::Cancel);
        }
        let toolbar = row![
            button("Open…").on_press(Message::PickInput),
            button("Save as…").on_press(Message::PickOutput),
            run,
            cancel,
        ]
        .spacing(8)
        .wrap();
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
                            text(hint),
                            text("Input path"),
                            row![
                                text_input("Path to local input", &self.input)
                                    .on_input(Message::Input)
                                    .padding(8)
                                    .style(input_style),
                                button("Browse…").on_press(Message::PickInput)
                            ]
                            .spacing(8)
                            .wrap(),
                            text("New output path"),
                            row![
                                text_input("Output path", &self.output)
                                    .on_input(Message::Output)
                                    .padding(8)
                                    .style(input_style),
                                button("Save as…").on_press(Message::PickOutput)
                            ]
                            .spacing(8)
                            .wrap(),
                            text("Operation parameter"),
                            text_input("See operation description", &self.parameter)
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
                    let panel = match self.workspace_tab {
                        WorkspaceTab::Result => column![
                            text("Result and warnings"),
                            text(&self.preview).wrapping(text::Wrapping::WordOrGlyph),
                        ]
                        .spacing(8),
                        WorkspaceTab::Draft => column![text("Editable draft / transcript"), {
                            let editor = text_editor(&self.editor).height(Length::Fill).style(
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
        container(column![toolbar, content].spacing(8).padding(8))
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
    ["Enter", "Escape"]
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

/// Open the workbench with theme and scale saved under a stable application key.
///
/// Keys use lowercase ASCII letters, digits and hyphens. The application stores only
/// its theme and scale in the user configuration directory. Input paths and drafts
/// are session data. When preferences are unavailable, the workbench uses defaults.
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
                    selected: ops.first().map(|o| o.id).unwrap_or("").into(),
                    input: String::new(),
                    input_selection: None,
                    output: String::new(),
                    output_selection: None,
                    parameter: String::new(),
                    editor: text_editor::Content::new(),
                    preview: String::new(),
                    preferences,
                    preferences_path,
                    preferences_error,
                    job: None,
                    status: "Ready".into(),
                    panes,
                    workspace_tab: WorkspaceTab::Result,
                },
                Task::none(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
