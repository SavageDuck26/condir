//! condir: the config folder for whatever package calls these macros.

use std::io;
use std::path::PathBuf;
// =========================================================================================================
// The package name that condir is being used in.
#[macro_export]
macro_rules! pack_name {
    () => {
        ::core::env!("CARGO_PKG_NAME")
    };
}
// =========================================================================================================
// Just returns the config dir: nothing is created, use `make_pack_config!` for that.

/// ```
/// let path = condir::config_path!("settings.json").unwrap();
/// assert!(path.ends_with("settings.json"));
/// ```
#[macro_export]
macro_rules! config_path {
    ($file:expr $(,)?) => {
        $crate::get_config_dir().map(|dir| dir.join($crate::pack_name!()).join($file))
    };
}
// =========================================================================================================
// Names the folders yourself from the config dir down, instead of using the package name:
// make_config!("crypt", "config.json") writes configdir/crypt/config.json and
// make_config!("config.json") writes configdir/config.json.
/// ```no_run
/// # fn main() -> std::io::Result<()> {
/// let made = condir::make_config!("crypt", "profiles", "config.json")?;
/// println!("{}", made.display());
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! make_config {
    ($($part:expr),+ $(,)?) => {
        $crate::get_config_dir()
            .map(|mut dir| {
                $(dir.push($part);)+
                dir
            })
            .and_then($crate::ensure_config_file)
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
        $crate::make_config!($crate::pack_name!(), $file)
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
