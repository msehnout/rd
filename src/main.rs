mod comparator;
mod error;

use error::Error;
use comparator::{FileMetadata, read_metadata, compare_files};

use docopt::Docopt;
use serde_json::{json, Value};
use walkdir::{WalkDir, DirEntry};
use xattr;

use std::collections::BTreeSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str;
use std::str::FromStr;

// See docopt documentation for more info about the format.
const USAGE: &'static str = r#"
Compare two filesystem trees. The first one is considered "the original" and the following one
"the new one". In will output JSON structure with differences in these two trees.

Usage: rd [-V] <fstree1> <fstree2>
       rd [-h]

Options:
    -V, --verbose  Produce huuuge amount of output (TODO)
    -h, --help     Print this help
"#;

fn main() {
    // Parse argv and exit the program with an error message if it fails.
    let args = Docopt::new(USAGE)
        .and_then(|d| d.argv(std::env::args().into_iter()).parse())
        .unwrap_or_else(|e| e.exit());

    if args.get_bool("-h") {
        print!("{}", USAGE);
        exit(0);
    }

    let (fstree1, fstree2) = match (PathBuf::from_str(args.get_str("<fstree1>")),
                                    PathBuf::from_str(args.get_str("<fstree2>")))
        {
            (Ok(a), Ok(b)) => (a, b),
            _ => {
                println!("One of the paths is not valid");
                exit(1);
            }
        };
    // TODO: let verbose = args.get_bool("-V");
    if let Err(e) = run(fstree1, fstree2) {
        eprintln!("{:?}", e);
    }
}

fn run(fstree1: PathBuf, fstree2: PathBuf) -> Result<(), Error> {
    let a: BTreeSet<_> = fstree_to_set(&fstree1)?;
    let b: BTreeSet<_> = fstree_to_set(&fstree2)?;

    let deleted_files = symmetric_difference(&a, &b);
    let added_files = symmetric_difference(&b, &a);

    let mut differences: Vec<Value> = Vec::new();

    for f in a.intersection(&b) {
        match compare_files(&fstree1, &fstree2, f) {
            Ok((false, difference)) => {
                differences.push(json!({
                    "name": f.to_str(),
                    "differences": difference
                }))
            }
            Err(e) => {
                println!("Failed on {}", f.display());
                return Err(e);
            }
            _ => {}
        }
    }

    let output = json!({
        "deleted_files": deleted_files,
        "added_files": added_files,
        "differences": differences
    });

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or("Failed to format JSON".into()));

    Ok(())
}

/// Take directory entry `de` and strip the `prefix`. This is a separate function just for clarity
/// of the `fstree_to_set`.
fn process_dir_entry(prefix: &Path, de: Result<walkdir::DirEntry, walkdir::Error>)
                     -> Result<PathBuf, Error>
{
    Ok(de?.path().strip_prefix(prefix)?.to_owned())
}

/// Take a filesystem tree and turn in into ordered (that's why BTree) set of files without leading
/// prefix.
fn fstree_to_set(fstree: &Path) -> Result<BTreeSet<PathBuf>, Error> {
    WalkDir::new(fstree)
        .into_iter()
        .map(|dir_entry| process_dir_entry(fstree, dir_entry))
        .collect()
}

/// Return a list of files present in the first set, but not in the second
fn symmetric_difference(first: &BTreeSet<PathBuf>, second: &BTreeSet<PathBuf>)
                        -> Vec<PathBuf>
{
    let diff_a_b = first - second;
    // to_owned clones all the paths, but since this function is expected to return a short list,
    // I don't think that matters
    diff_a_b.iter().map(ToOwned::to_owned).collect()
}
