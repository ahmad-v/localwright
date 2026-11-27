use iced::keyboard::{key::Named, Key, Modifiers};
use localwright_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_IMPORT_BYTES: usize = 16 * 1024;

/// A command exposed by the shared workbench shortcut registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutCommand {
    /// Run the selected operation.
    Run,
    /// Request cooperative cancellation of the active operation.
    Cancel,
    /// Reduce the controls pane width.
    NarrowControls,
    /// Increase the controls pane width.
    WidenControls,
    /// Open a new project tab.
    NewProject,
    /// Close the active project tab.
    CloseProject,
}

/// A platform-independent key used by a configurable workbench shortcut.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutKey {
    /// The Enter/Return key.
    Enter,
    /// The Escape key.
    Escape,
    /// An ASCII letter, digit, or pane-resize bracket.
    Character(char),
}

impl ShortcutKey {
    /// Return a stable key label suitable for preferences and user interfaces.
    pub fn label(&self) -> String {
        match self {
            Self::Enter => "Enter".into(),
            Self::Escape => "Escape".into(),
            Self::Character(character) => character.to_ascii_uppercase().to_string(),
        }
    }

    /// Parse a stable label from the shared workbench key selector.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "Enter" => Some(Self::Enter),
            "Escape" => Some(Self::Escape),
            _ if label.len() == 1 => {
                let character = label.chars().next()?;
                (character.is_ascii_alphanumeric() || matches!(character, '[' | ']'))
                    .then_some(Self::Character(character.to_ascii_lowercase()))
            }
            _ => None,
        }
    }
}

/// A key and modifier combination for a workbench command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortcutBinding {
    /// The key activated by this binding.
    pub key: ShortcutKey,
    /// Use Control on Windows/Linux and Command on macOS.
    #[serde(default)]
    pub command: bool,
    /// Require Shift to be held.
    #[serde(default)]
    pub shift: bool,
    /// Require Alt/Option to be held.
    #[serde(default)]
    pub alt: bool,
}

/// Validated bindings for common workbench commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortcutRegistry {
    bindings: BTreeMap<ShortcutCommand, ShortcutBinding>,
}

impl Default for ShortcutRegistry {
    fn default() -> Self {
        Self {
            bindings: BTreeMap::from([
                (
                    ShortcutCommand::Run,
                    ShortcutBinding {
                        key: ShortcutKey::Enter,
                        command: true,
                        shift: false,
                        alt: false,
                    },
                ),
                (
                    ShortcutCommand::Cancel,
                    ShortcutBinding {
                        key: ShortcutKey::Escape,
                        command: false,
                        shift: false,
                        alt: false,
                    },
                ),
                (
                    ShortcutCommand::NarrowControls,
                    ShortcutBinding {
                        key: ShortcutKey::Character('['),
                        command: true,
                        shift: false,
                        alt: true,
                    },
                ),
                (
                    ShortcutCommand::WidenControls,
                    ShortcutBinding {
                        key: ShortcutKey::Character(']'),
                        command: true,
                        shift: false,
                        alt: true,
                    },
                ),
                (
                    ShortcutCommand::NewProject,
                    ShortcutBinding {
                        key: ShortcutKey::Character('t'),
                        command: true,
                        shift: false,
                        alt: false,
                    },
                ),
                (
                    ShortcutCommand::CloseProject,
                    ShortcutBinding {
                        key: ShortcutKey::Character('w'),
                        command: true,
                        shift: false,
                        alt: false,
                    },
                ),
            ]),
        }
    }
}

impl ShortcutRegistry {
    /// Construct the default registry (Command/Ctrl+Enter, Escape).
    pub fn new() -> Self {
        Self::default()
    }

    /// Return the binding registered for a command.
    pub fn binding(&self, command: ShortcutCommand) -> Option<&ShortcutBinding> {
        self.bindings.get(&command)
    }

    /// Add newly introduced workbench defaults when reading older preferences.
    pub(super) fn add_missing_defaults(&mut self) {
        for (command, preferred) in ShortcutRegistry::default().bindings {
            if self.bindings.contains_key(&command) {
                continue;
            }
            let candidate = if !self.bindings.values().any(|current| current == &preferred) {
                Some(preferred.clone())
            } else {
                (0..8).find_map(|flags| {
                    let binding = ShortcutBinding {
                        key: preferred.key.clone(),
                        command: flags & 1 != 0,
                        shift: flags & 2 != 0,
                        alt: flags & 4 != 0,
                    };
                    (!self.bindings.values().any(|current| current == &binding)).then_some(binding)
                })
            };
            if let Some(binding) = candidate {
                self.bindings.insert(command, binding);
            }
        }
    }

    /// Register a binding, rejecting invalid keys and collisions atomically.
    pub fn set(&mut self, command: ShortcutCommand, binding: ShortcutBinding) -> Result<()> {
        let mut candidate = self.clone();
        candidate.bindings.insert(command, binding);
        candidate.validate()?;
        *self = candidate;
        Ok(())
    }

    /// Parse a bounded JSON customization and validate every binding before returning it.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_IMPORT_BYTES {
            return Err(Error::input("Shortcut customization exceeds 16 KiB"));
        }
        let registry: Self = serde_json::from_slice(bytes)?;
        registry.validate()?;
        Ok(registry)
    }

    /// Serialize the current registry as JSON for an application settings file.
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        Ok(serde_json::to_vec_pretty(self)?)
    }

    /// Resolve a native key event to a command, if a binding matches exactly.
    pub fn resolve(&self, key: &Key, modifiers: Modifiers) -> Option<ShortcutCommand> {
        self.bindings
            .iter()
            .find_map(|(command, binding)| binding.matches(key, modifiers).then_some(*command))
    }

    /// Validate the current bindings and reject collisions.
    pub fn validate(&self) -> Result<()> {
        if self.bindings.is_empty() || self.bindings.len() > 32 {
            return Err(Error::input(
                "Shortcut registry must contain between 1 and 32 bindings",
            ));
        }
        let mut combinations = Vec::with_capacity(self.bindings.len());
        for binding in self.bindings.values() {
            match &binding.key {
                ShortcutKey::Character(character)
                    if (!character.is_ascii_alphanumeric() && !matches!(character, '[' | ']'))
                        || character.is_ascii_uppercase() =>
                {
                    return Err(Error::input(
                        "Shortcut keys must be lowercase ASCII letters, digits or brackets",
                    ));
                }
                _ => {}
            }
            if combinations.contains(&binding) {
                return Err(Error::input("Shortcut bindings must be unique"));
            }
            combinations.push(binding);
        }
        Ok(())
    }

    /// Return a concise, platform-aware label suitable for help text.
    pub fn summary(&self) -> String {
        let mut labels = Vec::new();
        for command in [
            ShortcutCommand::Run,
            ShortcutCommand::Cancel,
            ShortcutCommand::NarrowControls,
            ShortcutCommand::WidenControls,
            ShortcutCommand::NewProject,
            ShortcutCommand::CloseProject,
        ] {
            if let Some(binding) = self.binding(command) {
                let name = match command {
                    ShortcutCommand::Run => "Run",
                    ShortcutCommand::Cancel => "Cancel",
                    ShortcutCommand::NarrowControls => "Narrow controls pane",
                    ShortcutCommand::WidenControls => "Widen controls pane",
                    ShortcutCommand::NewProject => "New project tab",
                    ShortcutCommand::CloseProject => "Close project tab",
                };
                let mut key = String::new();
                if binding.command {
                    key.push_str(if cfg!(target_os = "macos") {
                        "Command+"
                    } else {
                        "Ctrl+"
                    });
                }
                if binding.alt {
                    key.push_str(if cfg!(target_os = "macos") {
                        "Option+"
                    } else {
                        "Alt+"
                    });
                }
                if binding.shift {
                    key.push_str("Shift+");
                }
                key.push_str(&binding.key.label());
                labels.push(format!("{name}: {key}"));
            }
        }
        labels.join("  ·  ")
    }
}

impl ShortcutBinding {
    fn matches(&self, event_key: &Key, modifiers: Modifiers) -> bool {
        let key_matches = match (&self.key, event_key.as_ref()) {
            (ShortcutKey::Enter, Key::Named(Named::Enter)) => true,
            (ShortcutKey::Escape, Key::Named(Named::Escape)) => true,
            (ShortcutKey::Character(expected), Key::Character(actual)) => {
                actual.eq_ignore_ascii_case(&expected.to_string())
            }
            _ => false,
        };
        if !key_matches
            || modifiers.command() != self.command
            || modifiers.shift() != self.shift
            || modifiers.alt() != self.alt
        {
            return false;
        }
        if cfg!(target_os = "macos") {
            !modifiers.control()
        } else {
            !modifiers.logo()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_resolve_run_and_cancel_with_platform_command_semantics() {
        let registry = ShortcutRegistry::default();
        let enter = Key::Named(Named::Enter);
        let escape = Key::Named(Named::Escape);
        #[cfg(target_os = "macos")]
        let command = Modifiers::LOGO;
        #[cfg(not(target_os = "macos"))]
        let command = Modifiers::CTRL;
        assert_eq!(
            registry.resolve(&enter, command),
            Some(ShortcutCommand::Run)
        );
        assert_eq!(registry.resolve(&enter, Modifiers::empty()), None);
        assert_eq!(
            registry.resolve(&escape, Modifiers::empty()),
            Some(ShortcutCommand::Cancel)
        );
    }

    #[test]
    fn customization_import_is_bounded_validated_and_collision_safe() {
        let mut registry = ShortcutRegistry::default();
        let original = registry.clone();
        assert!(registry
            .set(
                ShortcutCommand::Run,
                ShortcutBinding {
                    key: ShortcutKey::Escape,
                    command: false,
                    shift: false,
                    alt: false,
                },
            )
            .is_err());
        assert_eq!(registry, original);
        let serialized = registry.to_json().unwrap();
        assert_eq!(ShortcutRegistry::from_json(&serialized).unwrap(), registry);
        assert!(ShortcutRegistry::from_json(&vec![b' '; MAX_IMPORT_BYTES + 1]).is_err());
        assert!(
            ShortcutRegistry::from_json(br#"{"bindings":{"run":{"key":{"character":"R"}}}}"#)
                .is_err()
        );
    }

    #[test]
    fn configurable_character_keys_match_native_events() {
        let mut registry = ShortcutRegistry::default();
        registry
            .set(
                ShortcutCommand::Run,
                ShortcutBinding {
                    key: ShortcutKey::from_label("R").unwrap(),
                    command: true,
                    shift: false,
                    alt: false,
                },
            )
            .unwrap();
        #[cfg(target_os = "macos")]
        let command = Modifiers::LOGO;
        #[cfg(not(target_os = "macos"))]
        let command = Modifiers::CTRL;
        assert_eq!(
            registry.resolve(&Key::Character("R".into()), command),
            Some(ShortcutCommand::Run)
        );
        assert_eq!(ShortcutKey::from_label("Escape"), Some(ShortcutKey::Escape));
        assert_eq!(ShortcutKey::from_label("?"), None);
    }

    #[test]
    fn default_panel_resize_shortcuts_resolve_and_survive_customization() {
        let registry = ShortcutRegistry::default();
        #[cfg(target_os = "macos")]
        let command = Modifiers::LOGO;
        #[cfg(not(target_os = "macos"))]
        let command = Modifiers::CTRL;
        let modifiers = command | Modifiers::ALT;
        assert_eq!(
            registry.resolve(&Key::Character("[".into()), modifiers),
            Some(ShortcutCommand::NarrowControls)
        );
        assert_eq!(
            registry.resolve(&Key::Character("]".into()), modifiers),
            Some(ShortcutCommand::WidenControls)
        );
        assert_eq!(
            ShortcutKey::from_label("["),
            Some(ShortcutKey::Character('['))
        );
        assert_eq!(
            ShortcutRegistry::from_json(&registry.to_json().unwrap()).unwrap(),
            registry
        );
    }
}
