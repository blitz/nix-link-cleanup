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

/// Return an iterator of all symlinks that point into the Nix store.
fn find_nix_store_links(
    root_directory: &Path,
    cross_filesystems: bool,
    verbose: bool,
) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(root_directory)
        .follow_links(false)
        .same_file_system(!cross_filesystems)
        .into_iter()
        // Ignore errors and keep going.
        .filter_map(move |maybe_entry| {
            maybe_entry
                .map_err(|err| {
                    if verbose {
                        eprintln!("Failed to walk: {err}");
                    }
                })
                .ok()
        })
        .filter(|e| e.path_is_symlink())
        // The symlink must look like a typical result link.
        .filter(move |e| {
            if let Some(file_name_str) = e.file_name().to_str() {
                let link_name_re = Regex::new(r"^(result|result-.+)$")
                    .expect("Failed to create regular expression");

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
        // It must point to the Nix store.
        .filter(move |e| {
            if let Ok(link_target) = fs::read_link(e.path()) {
                let nix_store_path = Path::new("/nix/store");

                link_target.starts_with(nix_store_path)
                    && (link_target
                        .strip_prefix(nix_store_path)
                        // This unwrap is safe, because we already
                        // checked whether the link starts with the
                        // right prefix.
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
}

fn main() {
    let args = Args::parse();

    for p in find_nix_store_links(&args.directory, args.cross_filesystems, args.verbosity != 0) {
        println!("{}", p.display());

        if args.delete {
            if let Err(e) = fs::remove_file(&p) {
                eprintln!("Failed to remove {}: {e}", p.display());
            }
        }
    }
}
