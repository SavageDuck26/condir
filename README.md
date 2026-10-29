# condir (config dir)

Folder for the config files of whatever package calls it: ask for the path, or ask for the path and have it made.

```toml
[dependencies]
condir = { git = "https://github.com/SavageDuck26/condir.git" }
```

```rust
use condir::{config_path, make_config};

fn main() -> std::io::Result<()> {
    // Nothing is created yet, this is only the path.
    let path = config_path!("settings.json")?;
    println!("{}", path.display());

    // The same path, with the folders and the file made if they were missing.
    let path = make_config!("settings.json")?;
    println!("created {}", path.display());
    Ok(())
}
```

In a package named `spm` those two give `~/.config/spm/settings.json`, and `make_config!` leaves an empty file behind for the caller to fill in with `std::fs::write` or an `OpenOptions` handle of its own.

## Where it points

| System | Folder | Read from |
|---|---|---|
| Windows | `%APPDATA%` | `APPDATA` |
| macOS | `~/Library/Application Support` | `HOME` |
| Everything else | `$XDG_CONFIG_HOME`, else `~/.config` | `XDG_CONFIG_HOME`, `HOME` |

The calling package's name is the folder inside that, so one crate cannot write into another one's space by accident.

## The package name

`pack_name!()` is the caller's `CARGO_PKG_NAME`. It has to be a macro because `env!` is substituted where it is *written*: inside a function it would bake in `condir` rather than your package. Two things follow from that:

- condir has to be compiled by Cargo. A build system that calls `rustc` directly fails with `environment variable 'CARGO_PKG_NAME' not defined` instead of quietly writing somewhere else.
- Inside condir's own tests and doc examples the name is `condir`. That is why the test for `make_config!` re-runs the test binary with a scratch `HOME`, `XDG_CONFIG_HOME` and `APPDATA` rather than writing into your real config folder.

## Items

| Item | What it gives |
|---|---|
| `config_path!("file")` | the path. Nothing is created. |
| `make_config!("file")` | the same path, with the folders and the file created |
| `pack_name!()` | the calling package's name, as `&'static str` |
| `get_config_dir()` | the system folder on its own, no package and no file |
| `ensure_config_file(path)` | the helper `make_config!` calls, hidden from the docs |

The two macros and the two functions all return `std::io::Result<PathBuf>`, so `?` works in a `main` that returns `Result`. Paths stay `PathBuf` and `OsString` the whole way, so a file name that is not valid UTF-8 survives intact. A variable that is missing or empty comes back as `NotFound` with a message that names it: `condir: APPDATA is not set`.

## Not done on purpose

- Only environment variables are read. A Windows `AppData` redirected through the registry, or a `HOME` changed by the macOS sandbox, is not seen.
- A `$XDG_CONFIG_HOME` that is relative is used as it is, where the base directory spec says to ignore it.
- `make_config!` creates and appends. It never truncates, and it never writes a byte of content.
- No dependencies, and nothing outside `std`.

## Misc

Edition 2024, so Rust 1.85 or newer. `cargo test` covers the folder rule of whatever system it runs on, the composed path, the error message, `ensure_config_file`, and both macros. The Windows and macOS branches are only compile-checked from here, never run.

## License

MIT, see [LICENSE](LICENSE).
