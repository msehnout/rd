mod comparator;
mod error;

use error::Error;
use comparator::{FileMetadata, read_metadata, compare_files};

use docopt::Docopt;
use xattr;
use walkdir::{WalkDir, DirEntry};

use std::collections::BTreeSet;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::str;
use std::str::FromStr;

// See docopt documentation for more info about the format.
const USAGE: &'static str = "
Usage: rd [-V] <fstree1> <fstree2>
       rd [-h]

Options:
    -V, --verbose  Produce huuuge amount of output
    -h, --help     Print this help
";

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

    symmetric_difference(&a, &b);
    symmetric_difference(&b, &a);

    for f in a.intersection(&b) {
        match compare_files(&fstree1, &fstree2, f) {
            Ok((false, f1, f2)) => {
                println!("Different metadata for file: {}", f.display());
                println!("f1: {:?}", f1);
                println!("f2: {:?}", f2);
            },
            Err(e) => {
                println!("Failed on {}", f.display());
                return Err(e);
            }
            _ => {}
        }
    }

    Ok(())
}

fn process_dir_entry(prefix: &Path, de: Result<walkdir::DirEntry, walkdir::Error>) -> Result<PathBuf, Error> {
    Ok(de?.path().strip_prefix(prefix)?.to_owned())
}

fn fstree_to_set(fstree: &Path) -> Result<BTreeSet<PathBuf>, Error> {
    WalkDir::new(fstree)
        .into_iter()
        .map(|dir_entry| process_dir_entry(fstree, dir_entry))
        .collect()
}

fn symmetric_difference(a: &BTreeSet<PathBuf>, b: &BTreeSet<PathBuf>) {
    let diff_a_b = a - b;
    println!("### A contains, but B does not:");
    diff_a_b.iter().for_each(|i| println!("{}", i.display()));
    println!("###");
}
