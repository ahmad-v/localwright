//! Explicit, versioned workspace session files for restoring operation context.
//!
//! Session files contain paths, parameters and drafts selected by the user. They
//! never execute operations while loading, and they deliberately omit result
//! payloads and loaded table data.

use localwright_core::{read_bounded, Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};

pub const MAX_SESSION_PROJECTS: usize = 32;
const MAX_SESSION_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TITLE_BYTES: usize = 256;
const MAX_TITLE_CHARS: usize = 64;
const MAX_OPERATION_BYTES: usize = 128;
const MAX_PATH_BYTES: usize = 4096;
const MAX_PARAMETER_BYTES: usize = 8192;
const MAX_DRAFT_BYTES: usize = 128 * 1024;

/// A native workbench layout captured in a user-selected session file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceLayout {
    /// Width of the controls pane as a percentage of the full workbench.
    pub controls_ratio: u8,
    /// Whether the common action toolbar is visible.
    pub toolbar_visible: bool,
}

/// The selected native workspace view for a saved project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceView {
    Results,
    Draft,
}

/// Operation context belonging to one tab in a saved session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceProject {
    /// User-facing project tab title.
    pub title: String,
    /// Stable operation identifier from the consuming application.
    pub operation: String,
    /// User-selected input path, if any.
    pub input: String,
    /// User-selected output path, if any.
    pub output: String,
    /// Operation-specific parameter text.
    pub parameter: String,
    /// Editable draft or transcript text.
    pub draft: String,
    /// The restored view. Table results are intentionally not persisted.
    pub view: WorkspaceView,
}

/// A bounded, versioned snapshot that can be moved between sessions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceSession {
    /// Session format version. Unknown versions are rejected, never guessed.
    pub schema_version: u32,
    /// Shared workbench layout preferences.
    pub layout: WorkspaceLayout,
    /// Zero-based index of the active project tab.
    pub active_project: usize,
    /// Ordered project tabs, capped at [`MAX_SESSION_PROJECTS`].
    pub projects: Vec<WorkspaceProject>,
}

impl WorkspaceSession {
    /// Validate a session before use or serialization.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err(Error::input("Unsupported workspace session version"));
        }
        if !(20..=70).contains(&self.layout.controls_ratio) {
            return Err(Error::input("Workspace controls ratio must be 20–70%"));
        }
        if self.projects.is_empty() || self.projects.len() > MAX_SESSION_PROJECTS {
            return Err(Error::input(format!(
                "Workspace sessions must contain 1–{MAX_SESSION_PROJECTS} projects"
            )));
        }
        if self.active_project >= self.projects.len() {
            return Err(Error::input("Active project is outside the saved session"));
        }
        for project in &self.projects {
            if project.title.chars().count() > MAX_TITLE_CHARS {
                return Err(Error::input(format!(
                    "Project titles are limited to {MAX_TITLE_CHARS} characters"
                )));
            }
            validate_text(
                "project title",
                &project.title,
                MAX_TITLE_BYTES,
                false,
                false,
            )?;
            validate_text(
                "operation identifier",
                &project.operation,
                MAX_OPERATION_BYTES,
                true,
                false,
            )?;
            validate_text("input path", &project.input, MAX_PATH_BYTES, true, false)?;
            validate_text("output path", &project.output, MAX_PATH_BYTES, true, false)?;
            validate_text(
                "operation parameter",
                &project.parameter,
                MAX_PARAMETER_BYTES,
                true,
                true,
            )?;
            validate_text("project draft", &project.draft, MAX_DRAFT_BYTES, true, true)?;
        }
        Ok(())
    }

    /// Load and validate a user-selected session without executing any operation.
    pub fn load(path: &Path) -> Result<Self> {
        let session: Self = serde_json::from_slice(&read_bounded(path, MAX_SESSION_BYTES)?)?;
        session.validate()?;
        Ok(session)
    }

    /// Save atomically. An existing destination is replaced only if it is already
    /// a valid Localwright session; unrelated files and symbolic links are kept.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let bytes = serde_json::to_vec_pretty(self)?;
        if bytes.len() as u64 > MAX_SESSION_BYTES {
            return Err(Error::new(
                ErrorKind::Limit,
                "Workspace session exceeds the 8 MiB limit",
            ));
        }

        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(Error::input(
                    "Workspace session destination must be a regular, non-symlink file",
                ));
            }
            Ok(_) => {
                // Never replace arbitrary content if a save dialog selected it.
                Self::load(path)?;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)?;
        let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
        temporary.write_all(&bytes)?;
        temporary.write_all(b"\n")?;
        temporary.as_file().sync_all()?;
        temporary.persist(path).map_err(|error| error.error)?;
        Ok(())
    }
}

fn validate_text(
    label: &str,
    value: &str,
    max_bytes: usize,
    allow_empty: bool,
    allow_multiline: bool,
) -> Result<()> {
    if (!allow_empty && value.trim().is_empty()) || value.len() > max_bytes {
        return Err(Error::input(format!("Invalid or oversized {label}")));
    }
    if value.chars().any(|character| {
        character.is_control() && !(allow_multiline && matches!(character, '\n' | '\r' | '\t'))
    }) {
        return Err(Error::input(format!(
            "{label} contains a control character"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> WorkspaceSession {
        WorkspaceSession {
            schema_version: 1,
            layout: WorkspaceLayout {
                controls_ratio: 42,
                toolbar_visible: true,
            },
            active_project: 1,
            projects: vec![
                WorkspaceProject {
                    title: "First".into(),
                    operation: "inspect".into(),
                    input: "/data/input.csv".into(),
                    output: String::new(),
                    parameter: String::new(),
                    draft: String::new(),
                    view: WorkspaceView::Results,
                },
                WorkspaceProject {
                    title: "Second".into(),
                    operation: "transform".into(),
                    input: "/data/other.csv".into(),
                    output: "/data/output.csv".into(),
                    parameter: "{\"column\":\"value\"}".into(),
                    draft: "draft text\n".into(),
                    view: WorkspaceView::Draft,
                },
            ],
        }
    }

    #[test]
    fn session_round_trip_preserves_context_layout_and_active_tab() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("session.lw-session.json");
        let expected = sample();
        expected.save(&path).unwrap();
        assert_eq!(WorkspaceSession::load(&path).unwrap(), expected);
    }

    #[test]
    fn save_updates_only_a_valid_session_and_preserves_unrecognized_files() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("session.json");
        let mut session = sample();
        session.save(&path).unwrap();
        session.active_project = 0;
        session.save(&path).unwrap();
        assert_eq!(WorkspaceSession::load(&path).unwrap(), session);

        fs::write(&path, b"not a Localwright session").unwrap();
        assert!(session.save(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), b"not a Localwright session");
    }

    #[test]
    fn rejects_unknown_schema_invalid_active_tab_and_unbounded_text() {
        let mut session = sample();
        session.schema_version = 2;
        assert!(session.validate().is_err());
        let mut session = sample();
        session.active_project = session.projects.len();
        assert!(session.validate().is_err());
        let mut session = sample();
        session.projects[0].draft = "x".repeat(MAX_DRAFT_BYTES + 1);
        assert!(session.validate().is_err());
    }

    #[test]
    #[cfg(unix)]
    fn refuses_symlink_session_targets() {
        use std::os::unix::fs::symlink;
        let directory = tempfile::tempdir().unwrap();
        let target = directory.path().join("real.json");
        fs::write(&target, serde_json::to_vec(&sample()).unwrap()).unwrap();
        let link = directory.path().join("linked.json");
        symlink(&target, &link).unwrap();
        assert!(sample().save(&link).is_err());
        assert_eq!(WorkspaceSession::load(&target).unwrap(), sample());
    }
}
