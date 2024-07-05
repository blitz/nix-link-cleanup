use clap::{ArgAction, Parser};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A program to cleanup Nix result links.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The directory to recursively scan for Nix result links.
    directory: PathBuf,

    /// Whether to delete or only print the found links.
    #[arg(short = 'd', long)]
    delete: bool,

    /// Whether to cross filesystem boundaries.
    #[arg(short = 'x', long)]
    cross_filesystems: bool,

    /// Be more verbose. This prints errors that occur, which are otherwise silent.
    #[arg(short = 'v', long, action = ArgAction::Count)]
    verbosity: u8,
}

fn find_problematic_links(
    root_directory: &Path,
    cross_filesystems: bool,
    verbose: bool,
) -> Vec<PathBuf> {
    let link_name_re =
        Regex::new(r"^(result|result-.+)$").expect("Failed to create regular expression");
    let nix_store_path = Path::new("/nix/store");

    WalkDir::new(root_directory)
        .follow_links(false)
        .same_file_system(!cross_filesystems)
        .into_iter()
        .filter_map(|maybe_entry| {
            maybe_entry
                .map_err(|err| {
                    if verbose {
                        eprintln!("Failed to walk: {err}");
                    }
                    ()
                })
                .ok()
        })
        .filter(|e| e.path_is_symlink())
        // The symlink looks like a typical result link?
        .filter(|e| {
            if let Some(file_name_str) = e.file_name().to_str() {
                link_name_re.is_match(file_name_str)
            } else {
                if verbose {
                    eprintln!(
                        "Invalid UTF-8 in name. Ignoring: {}",
                        e.file_name().to_string_lossy()
                    );
                }
                false
            }
        })
        // It points to the Nix store?
        .filter(|e| {
            if let Ok(link_target) = fs::read_link(e.path()) {
                link_target.starts_with(nix_store_path)
                    && (link_target
                        .strip_prefix(nix_store_path)
                        .unwrap()
                        .components()
                        .count()
                        == 1)
            } else {
                if verbose {
                    eprintln!(
                        "Failed to read link target. Ignoring: {}",
                        e.path().to_string_lossy()
                    );
                }
                false
            }
        })
        // We only want to remember the path.
        .map(|e| e.path().to_path_buf())
        .collect::<Vec<_>>()
}

fn main() {
    let args = Args::parse();

    for p in find_problematic_links(&args.directory, args.cross_filesystems, args.verbosity != 0) {
        println!("{}", p.display());

        if args.delete {
            if let Err(e) = fs::remove_file(&p) {
                eprintln!("Failed to remove {}: {e}", p.display());
            }
        }
    }
}
