use std::fs::{self, read_link, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::os::unix::fs::MetadataExt;

use super::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct FileMetadata {
    mode: u32,
    uid: u32,
    gid: u32,
    size: u64,
    selinux_label: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct DirMetadata {
    mode: u32,
    uid: u32,
    gid: u32,
    selinux_label: Option<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum FileType {
    File(FileMetadata),
    Dir(DirMetadata),
    Symlink(PathBuf),
}

pub fn read_metadata(file: &Path) -> Result<FileType, Error> {
    // Query the metadata about a file without following symlinks!
    let metadata = fs::symlink_metadata(file)?;

    if metadata.file_type().is_symlink() {
        // println!("{} is a symlink", file.display());
        Ok(FileType::Symlink(read_link(file)?))
    } else if metadata.file_type().is_dir() {
        Ok(FileType::Dir(DirMetadata {
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            selinux_label: read_selinux_label(file)?,
        }))
    } else {
        Ok(FileType::File(FileMetadata {
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.len(),
            selinux_label: read_selinux_label(file)?,
        }))
    }
}

fn read_file_content(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    Ok(data)
}

pub fn compare_files(prefix1: &PathBuf, prefix2: &PathBuf, file: &Path)
    -> Result<(bool, FileType, FileType), Error> {
    let f1 = prefix1.join(file);
    // println!("{}", f1.display());
    let f2 = prefix2.join(file);
    // println!("{}", f2.display());
    let m1 = read_metadata(&f1)?;
    let m2 = read_metadata(&f2)?;
    match (&m1, &m2) {
        (FileType::File(_), FileType::File(_)) => {
            let c1 = read_file_content(&f1)?;
            let c2 = read_file_content(&f2)?;
            Ok((m1 == m2 && c1 == c2, m1, m2))
        },
        _ => Ok((m1 == m2, m1, m2)),
    }

}

fn read_selinux_label(file: &Path) -> Result<Option<String>, Error> {
    let attribute = "security.selinux";
    if let Ok(attr) = xattr::get(file, attribute) {
        if let Some(label_bytes) = attr {
            if let Ok(label_str) = str::from_utf8(&label_bytes) {
                //println!("SELinux label: {}", label_str);
                Ok(Some(label_str.into()))
            } else {
                Err(Error::SELinux("SELinux label is not valid UTF-8"))
            }
        } else {
            //Err("SELinux label is empty.");
            Ok(None)
        }
    } else {
        Err(Error::SELinux("Failed to read SELinux label."))
    }
}