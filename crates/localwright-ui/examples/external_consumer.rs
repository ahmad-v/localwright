use localwright_core::{Job, Operation, Outcome, Request, Result};
fn handle(r: Request, j: Job) -> Result<Outcome> {
    j.check()?;
    Ok(Outcome {
        preview: format!("{} Unicode characters", r.parameter.chars().count()),
        ..Outcome::default()
    })
}
fn main() -> iced::Result {
    localwright_ui::run(
        "Independent text counter",
        &[Operation {
            id: "count",
            label: "Count",
            hint: "Enter text in the parameter field",
        }],
        handle,
    )
}
