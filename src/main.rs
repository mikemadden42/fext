use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const USAGE: &str = "usage: fext [DIR]";

/// Grouping key for a file. `Missing` sorts before any `Named` extension, so
/// files without an extension are listed first.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Extension {
    Missing,
    Named(String),
}

impl fmt::Display for Extension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Extension::Missing => f.write_str("(none)"),
            Extension::Named(ext) => f.write_str(ext),
        }
    }
}

/// Returns the lowercased extension of `name`. A trailing dot (`"file."`)
/// counts as no extension.
fn extension_of(name: &str) -> Extension {
    match Path::new(name).extension() {
        Some(ext) if !ext.is_empty() => Extension::Named(ext.to_string_lossy().to_lowercase()),
        _ => Extension::Missing,
    }
}

/// Groups file names by extension, skipping hidden files. Names within each
/// group are sorted case-insensitively.
fn group_by_extension<I>(names: I) -> BTreeMap<Extension, Vec<String>>
where
    I: IntoIterator<Item = String>,
{
    let mut groups: BTreeMap<Extension, Vec<String>> = BTreeMap::new();
    for name in names {
        if name.starts_with('.') {
            continue;
        }
        groups.entry(extension_of(&name)).or_default().push(name);
    }
    for names in groups.values_mut() {
        names.sort_by_cached_key(|name| (name.to_lowercase(), name.clone()));
    }
    groups
}

/// Lists the names of regular files in `dir`, including symlinks to files.
/// Names that aren't valid UTF-8 are converted lossily; broken symlinks are
/// reported on stderr and skipped.
fn list_files(dir: &Path) -> Result<Vec<String>, String> {
    let read_error = |e| format!("cannot read {}: {e}", dir.display());
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        let path = entry.path();
        if path.is_file() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        } else if path.is_symlink() && !path.exists() {
            eprintln!("fext: skipping broken symlink {}", path.display());
        }
    }
    Ok(names)
}

fn print_groups(groups: &BTreeMap<Extension, Vec<String>>) {
    for (extension, names) in groups {
        println!("{extension}:");
        for name in names {
            println!("- {name}");
        }
        println!();
    }
}

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let dir = match (args.next(), args.next()) {
        (None, _) => PathBuf::from("."),
        (Some(arg), None) if arg == "-h" || arg == "--help" => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        (Some(arg), None) => PathBuf::from(arg),
        (Some(_), Some(_)) => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match list_files(&dir) {
        Ok(names) => {
            print_groups(&group_by_extension(names));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("fext: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(ext: &str) -> Extension {
        Extension::Named(ext.to_string())
    }

    fn group(names: &[&str]) -> Vec<(Extension, Vec<String>)> {
        group_by_extension(names.iter().map(|n| n.to_string()))
            .into_iter()
            .collect()
    }

    #[test]
    fn extension_is_lowercased() {
        assert_eq!(extension_of("a.TXT"), named("txt"));
    }

    #[test]
    fn last_extension_wins() {
        assert_eq!(extension_of("x.tar.gz"), named("gz"));
    }

    #[test]
    fn no_extension_and_trailing_dot_are_missing() {
        assert_eq!(extension_of("Makefile"), Extension::Missing);
        assert_eq!(extension_of("trailing."), Extension::Missing);
    }

    #[test]
    fn missing_extension_header() {
        assert_eq!(Extension::Missing.to_string(), "(none)");
    }

    #[test]
    fn missing_group_comes_first() {
        let groups = group(&["b.txt", "7z.7z", "Makefile", "trailing."]);
        let keys: Vec<_> = groups.iter().map(|(k, _)| k).collect();
        assert_eq!(keys, [&Extension::Missing, &named("7z"), &named("txt")]);
        assert_eq!(groups[0].1, ["Makefile", "trailing."]);
    }

    #[test]
    fn hidden_files_are_skipped() {
        assert!(group(&[".hidden", ".config.toml"]).is_empty());
    }

    #[test]
    fn names_sort_case_insensitively() {
        let groups = group(&["b.txt", "README.txt", "a.TXT"]);
        assert_eq!(groups[0].1, ["a.TXT", "b.txt", "README.txt"]);
    }
}
