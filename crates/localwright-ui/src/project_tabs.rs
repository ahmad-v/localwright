use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub(super) const MAX_PROJECT_TABS: usize = 32;
// Keep the next monotonic project id outside the invalid/overflow range.
pub(super) const MAX_PROJECT_ID: u64 = u64::MAX - (MAX_PROJECT_TABS as u64 + 1);
const MAX_TITLE_CHARS: usize = 64;

/// Bounded project-tab metadata persisted with the workbench layout.
/// Project inputs, drafts and results remain session-local.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PersistedProjectTabs {
    #[serde(default)]
    pub tabs: Vec<PersistedProjectTab>,
    #[serde(default)]
    pub active_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PersistedProjectTab {
    pub id: u64,
    pub title: String,
}

impl PersistedProjectTabs {
    pub fn validate(&self) -> Result<(), String> {
        if self.tabs.len() > MAX_PROJECT_TABS {
            return Err(format!(
                "At most {MAX_PROJECT_TABS} project tabs can be restored"
            ));
        }
        let mut ids = HashSet::with_capacity(self.tabs.len());
        for tab in &self.tabs {
            if tab.id == 0 || tab.id > MAX_PROJECT_ID || !ids.insert(tab.id) {
                return Err("Project tab identifiers must be unique and in range".into());
            }
            if tab.title.trim().is_empty()
                || tab.title.chars().count() > MAX_TITLE_CHARS
                || tab.title.chars().any(char::is_control)
            {
                return Err(format!(
                    "Project tab titles must contain 1–{MAX_TITLE_CHARS} printable characters"
                ));
            }
        }
        match (self.tabs.is_empty(), self.active_id) {
            (true, None) => Ok(()),
            (false, Some(active)) if ids.contains(&active) => Ok(()),
            _ => Err("The active project tab must identify a saved tab".into()),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.tabs.iter().map(|tab| tab.id).max().unwrap_or(0) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_tabs_keep_order_and_active_id() {
        let saved = PersistedProjectTabs {
            tabs: vec![
                PersistedProjectTab {
                    id: 4,
                    title: "Analysis".into(),
                },
                PersistedProjectTab {
                    id: 9,
                    title: "Report".into(),
                },
            ],
            active_id: Some(9),
        };
        saved.validate().unwrap();
        assert_eq!(saved.next_id(), 10);
        let encoded = serde_json::to_vec(&saved).unwrap();
        assert_eq!(
            serde_json::from_slice::<PersistedProjectTabs>(&encoded).unwrap(),
            saved
        );
    }

    #[test]
    fn persisted_tabs_reject_duplicate_ids_invalid_selection_and_unbounded_values() {
        let tab = |id, title: &str| PersistedProjectTab {
            id,
            title: title.into(),
        };
        let cases = [
            PersistedProjectTabs {
                tabs: vec![tab(1, "One"), tab(1, "Duplicate")],
                active_id: Some(1),
            },
            PersistedProjectTabs {
                tabs: vec![tab(1, "One")],
                active_id: Some(2),
            },
            PersistedProjectTabs {
                tabs: vec![tab(0, "Zero")],
                active_id: Some(0),
            },
            PersistedProjectTabs {
                tabs: vec![tab(MAX_PROJECT_ID + 1, "Near limit")],
                active_id: Some(MAX_PROJECT_ID + 1),
            },
            PersistedProjectTabs {
                tabs: vec![tab(1, &"x".repeat(MAX_TITLE_CHARS + 1))],
                active_id: Some(1),
            },
        ];
        assert!(cases.iter().all(|tabs| tabs.validate().is_err()));
        let too_many = PersistedProjectTabs {
            tabs: (1..=MAX_PROJECT_TABS + 1)
                .map(|id| tab(id as u64, "Project"))
                .collect(),
            active_id: Some(1),
        };
        assert!(too_many.validate().is_err());
    }

    #[test]
    fn identifier_headroom_prevents_overflow_after_restoring_near_limit_values() {
        let saved = PersistedProjectTabs {
            tabs: vec![
                PersistedProjectTab {
                    id: 1,
                    title: "First".into(),
                },
                PersistedProjectTab {
                    id: MAX_PROJECT_ID,
                    title: "Highest allowed identifier".into(),
                },
            ],
            active_id: Some(MAX_PROJECT_ID),
        };
        saved.validate().unwrap();
        assert_eq!(saved.next_id(), MAX_PROJECT_ID + 1);
    }
}
