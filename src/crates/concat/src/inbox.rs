// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer Abdullah and Concat contributors

//! A drop from another process: a folder in the config folder that the
//! window looks into twice a second, so a helper on the same machine - the
//! gallery bridge, a script - can put files into the project on screen,
//! which the automation server cannot (it refuses the project the window
//! has open; live mode is #337).
//!
//! `inbox/open.json` says which project is open, `{"project": "<folder>"}`,
//! and is gone while none is, so the helper knows where to copy. A request
//! is any other `inbox/*.json`, `{"paths": ["<file>", ...]}`: the window
//! takes it out of the folder and imports its paths exactly as if they had
//! been dropped on it - the same probe, the same notices. A request that
//! arrives with no project open waits for one.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::host::Shell;
use crate::panes::Msg;
use crate::panes::media_bin::MediaMsg;

/// The inbox, under the app's config folder.
pub fn dir(config: &Path) -> PathBuf {
    config.join("inbox")
}

const OPEN: &str = "open.json";

thread_local! {
    /// The project last written to `open.json`, so it is written on a change only.
    static ANNOUNCED: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// The paths in one request, or `None` for one that is not a request.
fn paths_in(text: &str) -> Option<Vec<PathBuf>> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let paths = value.get("paths")?.as_array()?;
    Some(
        paths
            .iter()
            .filter_map(|path| path.as_str())
            .map(PathBuf::from)
            .collect(),
    )
}

/// The requests waiting in `dir`, oldest first, each taken out of the folder.
fn take_requests(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path.file_name().is_some_and(|name| name != OPEN)
        })
        .map(|path| {
            let at = std::fs::metadata(&path)
                .and_then(|meta| meta.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            (at, path)
        })
        .collect();
    files.sort();
    let mut paths = Vec::new();
    for (_, file) in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        // Taken out first: a request the import trips on is not taken twice.
        if std::fs::remove_file(&file).is_err() {
            continue;
        }
        match paths_in(&text) {
            Some(found) => paths.extend(found),
            None => log::warn!("inbox: {} is not a request", file.display()),
        }
    }
    paths
}

/// Keeps `open.json` saying which project is open.
fn announce(dir: &Path, project: Option<&str>) {
    let changed = ANNOUNCED.with(|last| last.borrow().as_deref() != project);
    if !changed {
        return;
    }
    let file = dir.join(OPEN);
    let written = match project {
        Some(project) => std::fs::create_dir_all(dir).and_then(|()| {
            let body = serde_json::json!({ "project": project }).to_string();
            let staged = dir.join(".open.json.part");
            std::fs::write(&staged, body)?;
            std::fs::rename(&staged, &file)
        }),
        None => match std::fs::remove_file(&file) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        },
    };
    match written {
        Ok(()) => ANNOUNCED.with(|last| *last.borrow_mut() = project.map(str::to_owned)),
        Err(error) => log::warn!("inbox: {}: {error}", file.display()),
    }
}

/// One look into the inbox; what the window's timer runs.
pub fn tick() {
    Shell::with(|shell, app| {
        let (dir, project) = {
            let Ok(studio) = shell.studio.try_borrow() else {
                return;
            };
            let project = if studio.on_start {
                None
            } else {
                studio
                    .session
                    .as_ref()
                    .map(|session| session.path().to_owned())
            };
            (dir(&studio.host.dirs.config), project)
        };
        announce(&dir, project.as_deref());
        if project.is_none() {
            return;
        }
        let paths = take_requests(&dir);
        if paths.is_empty() {
            return;
        }
        log::info!("inbox: importing {} file(s)", paths.len());
        let Ok(mut studio) = shell.studio.try_borrow_mut() else {
            return;
        };
        studio.handle(Msg::Media(MediaMsg::Import(paths)));
        studio.refresh_art();
        drop(studio);
        shell.studio.borrow().publish(&app, &shell.models);
    });
}

/// Takes `open.json` away as the window closes, so no helper copies into a
/// project nobody has open.
pub fn close() {
    Shell::with(|shell, _| {
        if let Ok(studio) = shell.studio.try_borrow() {
            announce(&dir(&studio.host.dirs.config), None);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_lists_its_paths() {
        let paths = paths_in(r#"{"paths": ["/a/b.mp4", "/c.heic"]}"#).expect("a request");
        assert_eq!(paths, vec![PathBuf::from("/a/b.mp4"), PathBuf::from("/c.heic")]);
        assert!(paths_in(r#"{"project": "/x"}"#).is_none());
        assert!(paths_in("not json").is_none());
    }

    #[test]
    fn requests_are_taken_oldest_first_and_open_is_left() {
        let dir = std::env::temp_dir().join(format!("concat-inbox-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(OPEN), r#"{"project": "/p"}"#).unwrap();
        std::fs::write(dir.join("1.json"), r#"{"paths": ["/one"]}"#).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(dir.join("2.json"), r#"{"paths": ["/two"]}"#).unwrap();
        std::fs::write(dir.join("bad.json"), "{").unwrap();
        let paths = take_requests(&dir);
        assert_eq!(paths, vec![PathBuf::from("/one"), PathBuf::from("/two")]);
        let left: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|entry| entry.file_name())
            .collect();
        assert_eq!(left, vec![std::ffi::OsString::from(OPEN)]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
