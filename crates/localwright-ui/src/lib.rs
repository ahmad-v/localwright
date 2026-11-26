//! A small native operation workbench for independently developed applications.
#![forbid(unsafe_code)]
use iced::{
    theme::Palette,
    widget::{
        button, column, container, pick_list, row, scrollable, text, text_editor, text_input,
    },
    Border, Color, Element, Length, Task, Theme,
};
use localwright_core::{Error, ErrorKind, Handler, Job, Operation, Outcome, Request, Result};
use std::fmt;

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
    output: String,
    parameter: String,
    editor: text_editor::Content,
    preview: String,
    theme: String,
    scale: u16,
    job: Option<Job>,
    status: String,
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
    Run,
    Cancel,
    Done(Result<Outcome>),
}
impl State {
    fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::Select(v) => self.selected = v.into(),
            Message::Input(v) => self.input = v,
            Message::Output(v) => self.output = v,
            Message::Parameter(v) => self.parameter = v,
            Message::Edit(a) => self.editor.perform(a),
            Message::Theme(v) => self.theme = v,
            Message::Scale(v) => self.scale = v,
            Message::Cancel => {
                if let Some(j) = &self.job {
                    j.cancel();
                    self.status = "Cancellation requested; waiting for operation".into();
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
                    input: self.input.clone().into(),
                    output: self.output.clone().into(),
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
        }
        Task::none()
    }
    fn view(&self) -> Element<'_, Message> {
        let ops = self.ops.iter().map(OperationChoice).collect::<Vec<_>>();
        let themes = THEMES.iter().map(|t| t.0.to_owned()).collect::<Vec<_>>();
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
        scrollable(
            container(
                column![
                    row![
                        row![
                            text("Operation"),
                            pick_list(ops, selected, |choice| Message::Select(choice.0.id))
                                .style(picker_style)
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                        row![
                            text("Theme"),
                            pick_list(themes, Some(self.theme.clone()), Message::Theme)
                                .style(picker_style)
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                        row![
                            text("UI scale (%)"),
                            pick_list(SCALES, Some(self.scale), Message::Scale).style(picker_style)
                        ]
                        .spacing(8)
                        .align_y(iced::Alignment::Center),
                    ]
                    .spacing(16)
                    .wrap(),
                    text(hint),
                    text("Input path"),
                    text_input("Path to local input", &self.input)
                        .on_input(Message::Input)
                        .padding(8)
                        .style(input_style),
                    text("New output path"),
                    text_input("Output path", &self.output)
                        .on_input(Message::Output)
                        .padding(8)
                        .style(input_style),
                    text("Operation parameter"),
                    text_input("See operation description", &self.parameter)
                        .on_input(Message::Parameter)
                        .padding(8)
                        .style(input_style),
                    row![run, cancel].spacing(8).wrap(),
                    text(&self.status).wrapping(text::Wrapping::WordOrGlyph),
                    text("Result and warnings"),
                    text(&self.preview).wrapping(text::Wrapping::WordOrGlyph),
                    text("Editable draft / transcript"),
                    {
                        let editor = text_editor(&self.editor)
                            .height(Length::Fixed(160.0))
                            .style(|theme, status| {
                                let mut style = text_editor::default(theme, status);
                                style.border =
                                    control_border(theme, status == text_editor::Status::Focused);
                                style
                            });
                        if self.job.is_none() {
                            editor.on_action(Message::Edit)
                        } else {
                            editor
                        }
                    },
                ]
                .spacing(8),
            )
            .width(Length::Fill)
            .padding(16),
        )
        .height(Length::Fill)
        .into()
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

/// Run a native window with one bounded background operation at a time.
pub fn run(title: &'static str, ops: &'static [Operation], handler: Handler) -> iced::Result {
    iced::application(title, State::update, State::view)
        .theme(|s: &State| preset(&s.theme))
        .scale_factor(|s: &State| f64::from(s.scale) / 100.0)
        .run_with(move || {
            (
                State {
                    ops,
                    handler,
                    selected: ops.first().map(|o| o.id).unwrap_or("").into(),
                    input: String::new(),
                    output: String::new(),
                    parameter: String::new(),
                    editor: text_editor::Content::new(),
                    preview: String::new(),
                    theme: "Modern Light".into(),
                    scale: 100,
                    job: None,
                    status: "Ready".into(),
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
}
