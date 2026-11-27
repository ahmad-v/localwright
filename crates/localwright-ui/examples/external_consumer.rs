use localwright_core::{Job, Operation, Outcome, Request, Result};
fn handle(r: Request, j: Job) -> Result<Outcome> {
    j.check()?;
    Ok(Outcome {
        preview: format!("{} Unicode characters", r.parameter.chars().count()),
        ..Outcome::default()
    })
}
fn main() -> iced::Result {
    localwright_ui::run_with_saved_appearance(
        "Independent text counter",
        "localwright-text-counter",
        &[Operation {
            id: "count",
            label: "Count",
            hint: "Enter text, then use New project tab or Ctrl/Command+T to open independent workspaces. Use Ctrl/Command+W to close the active tab and Ctrl+Tab or Ctrl+Shift+Tab to switch tabs. Saved workspaces restore their tab order and selection; input and results remain session-only. The toolbar preference is also saved.",
        }],
        handle,
    )
}
