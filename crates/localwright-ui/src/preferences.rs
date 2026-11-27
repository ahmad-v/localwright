use super::{ShortcutRegistry, SCALES, THEMES};
use localwright_core::{read_bounded, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};

const MAX_BYTES: u64 = 4 * 1024;

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
}

fn default_panel_ratio() -> u8 {
    34
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            schema_version: 1,
            theme: "Modern Light".into(),
            scale: 100,
            panel_ratio: default_panel_ratio(),
            shortcuts: ShortcutRegistry::default(),
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
        let settings: Self = serde_json::from_slice(&read_bounded(path, MAX_BYTES)?)?;
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
}
