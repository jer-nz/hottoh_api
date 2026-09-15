//! Optional module features (`[features]`), changed while running from the web interface or
//! `POST /api/features` when `[http_api] edit_features` allows it, and saved to the configuration
//! file. Only the lines of the `[features]` section are rewritten: comments, other sections and
//! the layout of the file are kept.

use crate::hottoh::config::FeaturesConfig;
use log::info;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockReadGuard};

/// Section of the configuration file holding the features
const SECTION: &str = "features";

/// Why a change of features was refused
#[derive(Debug, PartialEq, Eq)]
pub enum FeatureChangeError {
    /// `[http_api] edit_features` is off
    Locked,
    /// Unknown feature name
    Unknown(String),
    /// The configuration file could not be read or written
    Save(String),
}

/// Current features, shared by every HTTP worker
#[derive(Debug)]
pub struct FeatureSettings {
    current: RwLock<FeaturesConfig>,
    /// Configuration file the changes are saved to, `None` without file (kept in memory only)
    file: Option<PathBuf>,
    editable: bool,
}

impl FeatureSettings {
    pub fn new(features: FeaturesConfig, file: Option<PathBuf>, editable: bool) -> Self {
        Self {
            current: RwLock::new(features),
            file,
            editable,
        }
    }

    /// Features as configured, not editable (tests)
    #[cfg(test)]
    pub fn fixed(features: FeaturesConfig) -> Self {
        Self::new(features, None, false)
    }

    fn read(&self) -> RwLockReadGuard<'_, FeaturesConfig> {
        self.current.read().unwrap_or_else(|p| p.into_inner())
    }

    /// Copy of the current features
    pub fn get(&self) -> FeaturesConfig {
        self.read().clone()
    }

    pub fn editable(&self) -> bool {
        self.editable
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// Applies the changes and saves them to the configuration file first: when the file cannot
    /// be written, nothing changes. Returns the new features.
    pub fn update(
        &self,
        changes: &BTreeMap<String, bool>,
    ) -> Result<FeaturesConfig, FeatureChangeError> {
        if !self.editable {
            return Err(FeatureChangeError::Locked);
        }
        // Held while the file is written, so that two changes cannot overwrite each other
        let mut current = self.current.write().unwrap_or_else(|p| p.into_inner());
        let mut next = current.clone();
        let mut changed = Vec::new();
        for (name, &value) in changes {
            let field = next
                .field_mut(name)
                .ok_or_else(|| FeatureChangeError::Unknown(name.clone()))?;
            if *field != value {
                *field = value;
                changed.push((name.as_str(), value));
            }
        }
        if changed.is_empty() {
            return Ok(next);
        }
        if let Some(file) = &self.file {
            let error =
                |e: std::io::Error| FeatureChangeError::Save(format!("{}: {}", file.display(), e));
            let text = std::fs::read_to_string(file).map_err(error)?;
            write_file(file, &update_ini(&text, SECTION, &changed)).map_err(error)?;
        }
        for (name, value) in &changed {
            info!(
                "Feature {} {} through the API{}",
                name,
                if *value { "enabled" } else { "disabled" },
                self.file.as_ref().map_or(
                    " (not saved: no configuration file)".into(),
                    |f| format!(", saved to {}", f.display())
                )
            );
        }
        *current = next.clone();
        Ok(next)
    }
}

/// Replaces the file with a temporary copy renamed over it, or writes it in place when the
/// directory is not writable (a configuration file owned by the service in `/etc`)
fn write_file(path: &Path, text: &str) -> std::io::Result<()> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);
    let renamed = std::fs::write(&temporary, text).and_then(|()| {
        // Keeps the permissions of the original file
        if let Ok(metadata) = std::fs::metadata(path) {
            std::fs::set_permissions(&temporary, metadata.permissions())?;
        }
        std::fs::rename(&temporary, path)
    });
    if renamed.is_err() {
        let _ = std::fs::remove_file(&temporary);
        std::fs::write(path, text)?;
    }
    Ok(())
}

/// Sets `key = value` lines of an INI section: existing lines are rewritten in place, missing
/// keys are added at the end of the section, and the section is added at the end of the file
/// when there is none. Everything else is kept as is.
pub fn update_ini(text: &str, section: &str, values: &[(&str, bool)]) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let mut found = vec![false; values.len()];
    let mut in_section = false;
    // Index after the last key or comment line of the section
    let mut section_end: Option<usize> = None;

    for (index, line) in lines.iter_mut().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_section = trimmed[1..trimmed.len() - 1]
                .trim()
                .eq_ignore_ascii_case(section);
            if in_section {
                section_end = Some(index + 1);
            }
            continue;
        }
        if !in_section || trimmed.is_empty() {
            continue;
        }
        section_end = Some(index + 1);
        if trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        let Some((key, _)) = trimmed.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if let Some(position) = values
            .iter()
            .position(|(name, _)| name.eq_ignore_ascii_case(key))
        {
            let indent = &line[..line.len() - line.trim_start().len()];
            let ending = if line.ends_with('\n') { eol } else { "" };
            *line = format!("{}{} = {}{}", indent, key, values[position].1, ending);
            found[position] = true;
        }
    }

    let missing: Vec<String> = values
        .iter()
        .zip(&found)
        .filter(|(_, found)| !**found)
        .map(|((name, value), _)| format!("{} = {}{}", name, value, eol))
        .collect();
    if missing.is_empty() {
        return lines.concat();
    }
    let at = match section_end {
        Some(at) => at,
        None => {
            if lines.last().is_some_and(|l| !l.ends_with('\n')) {
                lines.push(eol.into());
            }
            if !lines.is_empty() {
                lines.push(eol.into());
            }
            lines.push(format!("[{}]{}", section, eol));
            lines.len()
        }
    };
    // The line before may be the last one of a file without final newline
    if at > 0 && !lines[at - 1].ends_with('\n') {
        lines[at - 1].push_str(eol);
    }
    lines.splice(at..at, missing);
    lines.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{Config, File, FileFormat};

    const INI: &str = "[stove]\nip = 192.168.1.150\n\n[features]\n# Schedule\n\
                       chrono_schedule_read = true\n  clock_write=false\n# end\n\n[log]\nlevel = info\n";

    #[test]
    fn existing_keys_are_rewritten_in_place() {
        let text = update_ini(INI, "features", &[("clock_write", true)]);
        assert_eq!(
            text,
            INI.replace("  clock_write=false", "  clock_write = true")
        );
    }

    #[test]
    fn missing_keys_go_to_the_end_of_the_section() {
        let text = update_ini(
            INI,
            "Features",
            &[("wifi_scan", true), ("clock_write", true)],
        );
        assert!(text.contains("  clock_write = true\n# end\nwifi_scan = true\n\n[log]"));
        let config = Config::builder()
            .add_source(File::from_str(&text, FileFormat::Ini))
            .build()
            .unwrap();
        assert!(config.get_bool("features.wifi_scan").unwrap());
        assert_eq!(config.get_string("log.level").unwrap(), "info");
    }

    #[test]
    fn section_is_added_when_missing() {
        let text = update_ini("[stove]\r\nip = auto", "features", &[("clock_write", true)]);
        assert_eq!(
            text,
            "[stove]\r\nip = auto\r\n\r\n[features]\r\nclock_write = true\r\n"
        );
        assert_eq!(
            update_ini("", "features", &[("pin_write", false)]),
            "[features]\npin_write = false\n"
        );
        // Same key in another section: left alone
        let text = update_ini(
            "[other]\nclock_write = false\n",
            "features",
            &[("clock_write", true)],
        );
        assert!(
            text.starts_with("[other]\nclock_write = false\n\n[features]\nclock_write = true\n")
        );
    }

    #[test]
    fn changes_are_saved_then_applied() {
        let dir = std::env::temp_dir().join(format!("hottoh-features-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("config.ini");
        std::fs::write(&file, INI).unwrap();
        let settings = FeatureSettings::new(FeaturesConfig::default(), Some(file.clone()), true);

        let changes = BTreeMap::from([("clock_write".to_string(), true)]);
        assert!(settings.update(&changes).unwrap().clock_write);
        assert!(settings.get().clock_write);
        assert!(
            std::fs::read_to_string(&file)
                .unwrap()
                .contains("  clock_write = true\n")
        );

        let unknown = BTreeMap::from([("coffee".to_string(), true)]);
        assert_eq!(
            settings.update(&unknown),
            Err(FeatureChangeError::Unknown("coffee".into()))
        );

        // Unwritable file: nothing changes
        std::fs::remove_file(&file).unwrap();
        let changes = BTreeMap::from([("wifi_scan".to_string(), true)]);
        assert!(matches!(
            settings.update(&changes),
            Err(FeatureChangeError::Save(_))
        ));
        assert!(!settings.get().wifi_scan);
        std::fs::remove_dir_all(&dir).unwrap();

        let locked = FeatureSettings::fixed(FeaturesConfig::default());
        assert_eq!(locked.update(&changes), Err(FeatureChangeError::Locked));
    }
}
