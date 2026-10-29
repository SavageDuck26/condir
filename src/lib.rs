//! condir: the config folder for whatever package calls these macros.

use std::io;
use std::path::{Component, Path, PathBuf};
// =========================================================================================================
// The package name that condir is being used in.
#[macro_export]
macro_rules! pack_name {
    () => {
        ::core::env!("CARGO_PKG_NAME")
    };
}
// =========================================================================================================
// Returns the path but creates nothing, use `make_pack_config!` for that. A leading slash
// in the file name is not needed and changes nothing, it still starts in the package
// folder inside the config dir.
/// ```
/// let path = condir::config_path!("settings.json").unwrap();
/// assert!(path.ends_with("settings.json"));
/// ```
#[macro_export]
macro_rules! config_path {
    ($file:expr $(,)?) => {
        $crate::config_path_at(
            ::std::path::Path::new($crate::pack_name!()).join($crate::no_roots($file)),
        )
    };
}
// =========================================================================================================
// Takes one path and starts it in the config dir. The leading slash is not needed:
// make_config!("crypt/config.json") and make_config!("/crypt/config.json") both write
// configdir/crypt/config.json, and every part but the last one becomes a folder.
/// ```no_run
/// # fn main() -> std::io::Result<()> {
/// let made = condir::make_config!("crypt/profiles/config.json")?;
/// println!("{}", made.display());
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! make_config {
    ($path:expr $(,)?) => {
        $crate::make_config_at($path)
    };
}
// =========================================================================================================
// If called in an app named "crypt", then make_pack_config!("config.json") would write it to configdir/crypt/config.json
/// ```no_run
/// # fn main() -> std::io::Result<()> {
/// let made = condir::make_pack_config!("config.json")?;
/// println!("{}", made.display());
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! make_pack_config {
    ($file:expr $(,)?) => {
        $crate::make_config!(
            ::std::path::Path::new($crate::pack_name!()).join($crate::no_roots($file))
        )
    };
}
// =========================================================================================================
/// The config dir for active OS, as a result path.
/// - Windows: `%APPDATA%`
/// - MacOS: `~/Library/Application Support`
/// - Else: `$XDG_CONFIG_HOME`, else `~/.config`
pub fn get_config_dir() -> io::Result<PathBuf> {
    if cfg!(target_os = "windows") {
        env_path("APPDATA")
    } else if cfg!(target_os = "macos") {
        Ok(env_path("HOME")?.join("Library/Application Support"))
    } else {
        match std::env::var_os("XDG_CONFIG_HOME") {
            Some(dir) if !dir.is_empty() => Ok(PathBuf::from(dir)),
            _ => Ok(env_path("HOME")?.join(".config")),
        }
    }
}

/// The parts of `path` that mean something under the config folder. A root or a prefix is
/// dropped, so a leading slash changes nothing, while a `..` still steps back out.
#[doc(hidden)]
pub fn no_roots(path: impl AsRef<Path>) -> PathBuf {
    let mut kept = PathBuf::new();

    for part in path.as_ref().components() {
        match part {
            Component::RootDir | Component::Prefix(_) | Component::CurDir => {}
            part => kept.push(part.as_os_str()),
        }
    }

    kept
}

/// `path` read from the config folder, creating nothing.
#[doc(hidden)]
pub fn config_path_at(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    Ok(get_config_dir()?.join(no_roots(path)))
}

/// The same, with every folder on the way and the file itself created.
#[doc(hidden)]
pub fn make_config_at(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    config_path_at(path).and_then(ensure_config_file)
}

/// Creates the parent folders if they are missing, touches the file, and hands the path back.
#[doc(hidden)]
pub fn ensure_config_file(path: PathBuf) -> io::Result<PathBuf> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    Ok(path)
}

fn env_path(name: &str) -> io::Result<PathBuf> {
    std::env::var_os(name).map(PathBuf::from).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("condir: {name} is not set"),
        )
    })
}
