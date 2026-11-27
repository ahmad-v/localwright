use super::{project_tabs::PersistedProjectTabs, ShortcutRegistry, SCALES, THEMES};
use localwright_core::{read_bounded, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 16 * 1024;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Preferences {
    schema_version: u32,
    pub theme: String,
    pub scale: u16,
    /// Sidebar width as a percentage of the available workspace width.
    #[serde(default = "default_panel_ratio")]
    pub panel_ratio: u8,
    #[serde(default)]
    pub shortcuts: ShortcutRegistry,
    /// Whether the common action toolbar is shown.
    #[serde(default = "default_toolbar_visible")]
    pub toolbar_visible: bool,
    /// Ordered project-tab metadata and active selection; tab content stays in memory.
    #[serde(default)]
    pub project_tabs: PersistedProjectTabs,
}

fn default_panel_ratio() -> u8 {
    34
}

fn default_toolbar_visible() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 1,
            theme: "Modern Light".into(),
            scale: 100,
            panel_ratio: default_panel_ratio(),
            shortcuts: ShortcutRegistry::default(),
            toolbar_visible: default_toolbar_visible(),
            project_tabs: PersistedProjectTabs::default(),
        }
    }
}

impl Preferences {
    pub fn load(path: &Path) -> Result<Self> {
        match fs::symlink_metadata(path) {
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Self::default()),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let mut settings: Self = serde_json::from_slice(&read_bounded(path, MAX_BYTES)?)?;
        settings.shortcuts.add_missing_defaults();
        settings.validate()?;
        Ok(settings)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        // Refuse to replace an existing unrecognized settings document.
        Self::load(path)?;
        let parent = path
            .parent()
            .ok_or_else(|| Error::input("Preferences path has no parent directory"))?;
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut file, self)?;
        file.write_all(b"\n")?;
        file.as_file().sync_all()?;
        file.persist(path).map_err(|error| error.error)?;
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !THEMES.iter().any(|preset| preset.0 == self.theme)
            || !SCALES.contains(&self.scale)
            || !(20..=70).contains(&self.panel_ratio)
        {
            return Err(Error::input("Unsupported workbench preferences"));
        }
        self.shortcuts.validate()?;
        self.project_tabs.validate().map_err(Error::input)?;
        Ok(())
    }
}

pub(super) fn app_preferences_path(app: &str) -> Option<PathBuf> {
    if app.is_empty()
        || !app
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return None;
    }
    let absolute_env = |key| {
        env::var_os(key)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    #[cfg(target_os = "windows")]
    let config = absolute_env("APPDATA");
    #[cfg(target_os = "macos")]
    let config = absolute_env("HOME").map(|home| home.join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let config = absolute_env("XDG_CONFIG_HOME")
        .or_else(|| absolute_env("HOME").map(|home| home.join(".config")));
    Some(config?.join(app).join("workbench.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_appearance_and_preserves_unrecognized_files() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        assert_eq!(Preferences::load(&path).unwrap(), Preferences::default());
        let mut prefs = Preferences {
            theme: THEMES[1].0.into(),
            scale: 200,
            ..Preferences::default()
        };
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap(), prefs);
        prefs.scale = 125;
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap(), prefs);

        let newer = br#"{"schema_version":2,"theme":"Other","scale":300}"#;
        fs::write(&path, newer).unwrap();
        assert!(Preferences::load(&path).is_err());
        assert!(prefs.save(&path).is_err());
        assert_eq!(fs::read(path).unwrap(), newer);
    }

    #[test]
    fn rejects_invalid_application_keys() {
        assert!(app_preferences_path("../other").is_none());
        assert!(app_preferences_path("").is_none());
        assert!(app_preferences_path("directory/app").is_none());
    }

    #[test]
    fn loads_v1_appearance_preferences_without_shortcuts() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            br#"{"schema_version":1,"theme":"Modern Light","scale":100}"#,
        )
        .unwrap();
        let loaded = Preferences::load(&path).unwrap();
        assert_eq!(loaded.shortcuts, ShortcutRegistry::default());
        assert_eq!(loaded.panel_ratio, 34);
        assert!(loaded.toolbar_visible);
        assert!(loaded.project_tabs.tabs.is_empty());
    }

    #[test]
    fn adds_new_panel_shortcuts_to_existing_shortcut_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            br#"{"schema_version":1,"theme":"Modern Light","scale":100,"shortcuts":{"bindings":{"run":{"key":"enter","command":true},"cancel":{"key":"escape"}}}}"#,
        )
        .unwrap();
        let loaded = Preferences::load(&path).unwrap();
        assert!(loaded
            .shortcuts
            .binding(super::super::ShortcutCommand::NarrowControls)
            .is_some());
        assert!(loaded
            .shortcuts
            .binding(super::super::ShortcutCommand::WidenControls)
            .is_some());
        assert!(loaded
            .shortcuts
            .binding(super::super::ShortcutCommand::NewProject)
            .is_some());
        assert!(loaded
            .shortcuts
            .binding(super::super::ShortcutCommand::CloseProject)
            .is_some());
    }

    #[test]
    fn validates_persisted_workspace_panel_bounds() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        let mut prefs = Preferences {
            panel_ratio: 71,
            ..Preferences::default()
        };
        assert!(prefs.save(&path).is_err());
        prefs.panel_ratio = 20;
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap().panel_ratio, 20);
        prefs.panel_ratio = 70;
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap().panel_ratio, 70);
    }

    #[test]
    fn persists_toolbar_visibility_and_migrates_older_preferences() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        let prefs = Preferences {
            toolbar_visible: false,
            ..Preferences::default()
        };
        prefs.save(&path).unwrap();
        assert!(!Preferences::load(&path).unwrap().toolbar_visible);

        fs::write(
            &path,
            br#"{"schema_version":1,"theme":"Modern Light","scale":100}"#,
        )
        .unwrap();
        assert!(Preferences::load(&path).unwrap().toolbar_visible);
    }

    #[test]
    fn persists_valid_project_tab_order_and_active_selection() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("app/workbench.json");
        let prefs = Preferences {
            project_tabs: PersistedProjectTabs {
                tabs: vec![
                    super::super::PersistedProjectTab {
                        id: 2,
                        title: "Draft".into(),
                    },
                    super::super::PersistedProjectTab {
                        id: 5,
                        title: "Review".into(),
                    },
                ],
                active_id: Some(5),
            },
            ..Preferences::default()
        };
        prefs.save(&path).unwrap();
        assert_eq!(Preferences::load(&path).unwrap(), prefs);
    }
}
