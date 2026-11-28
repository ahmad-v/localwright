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
            hint: "Enter text, then use New project tab or Ctrl/Command+T to open independent workspaces. Use Ctrl/Command+W to close the active tab and Ctrl+Tab or Ctrl+Shift+Tab to switch tabs. Enable High-contrast focus outlines in the controls to strengthen focus on editable fields. Saved appearance preferences restore tab metadata. Use File > Save workspace session… to explicitly save paths and drafts; Restore workspace session… appends tabs and never runs an operation automatically. Session files omit result payloads and are unencrypted.",
        }],
        handle,
    )
}
