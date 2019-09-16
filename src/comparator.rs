use std::fs::{self, read_link, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str;
use std::os::unix::fs::MetadataExt;

use super::error::Error;

trait ListDifferences {
    fn list_differences(&self, other: &Self) -> Vec<(String, String)>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct FileMetadata {
    mode: u32,
    uid: u32,
    gid: u32,
    size: u64,
    selinux_label: Option<String>,
}

impl ListDifferences for FileMetadata {
    fn list_differences(&self, other: &FileMetadata) -> Vec<(String, String)> {
        let mut ret = Vec::new();
        if self.mode != other.mode {
           ret.push(("mode".into(), self.mode.to_string()))
        }
        if self.uid != other.uid {
            ret.push(("uid".into(), self.uid.to_string()))
        }
        if self.gid != other.gid {
            ret.push(("gid".into(), self.gid.to_string()))
        }
        if self.size != other.size {
            ret.push(("size".into(), self.size.to_string()))
        }
        if self.selinux_label != other.selinux_label {
            ret.push(("selinux_label".into(), self.selinux_label.clone().unwrap_or("".into())))
        }
        ret
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct DirMetadata {
    mode: u32,
    uid: u32,
    gid: u32,
    selinux_label: Option<String>,
}

impl ListDifferences for DirMetadata {
    fn list_differences(&self, other: &DirMetadata) -> Vec<(String, String)> {
        let mut ret = Vec::new();
        if self.mode != other.mode {
            ret.push(("mode".into(), self.mode.to_string()))
        }
        if self.uid != other.uid {
            ret.push(("uid".into(), self.uid.to_string()))
        }
        if self.gid != other.gid {
            ret.push(("gid".into(), self.gid.to_string()))
        }
        if self.selinux_label != other.selinux_label {
            ret.push(("selinux_label".into(), self.selinux_label.clone().unwrap_or("".into())))
        }
        ret
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FileType {
    File(FileMetadata),
    Dir(DirMetadata),
    Symlink(PathBuf),
}

/// Load metadata about the file.
pub fn read_metadata(file: &Path) -> Result<FileType, Error> {
    // Query the metadata about a file without following symlinks!
    let metadata = fs::symlink_metadata(file)?;

    if metadata.file_type().is_symlink() {
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

/// Load file content into a byte buffer
fn read_file_content(path: &Path) -> Result<Vec<u8>, Error> {
    let mut file = File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(data)
}

/// Compare single file in two filesystem trees. In case it is a regular file, compare its content
/// as well.
pub fn compare_files(prefix1: &PathBuf, prefix2: &PathBuf, file: &Path)
    -> Result<(bool, Vec<(String, String)>), Error> {
    let f1 = prefix1.join(file);
    let f2 = prefix2.join(file);
    let type1 = read_metadata(&f1)?;
    let type2 = read_metadata(&f2)?;
    match (&type1, &type2) {
        // For regular file, compare their content
        (FileType::File(m1), FileType::File(m2)) => {
            let c1 = read_file_content(&f1)?;
            let c2 = read_file_content(&f2)?;
            let same_content = c1 == c2;
            if m1 == m2 && same_content {
                Ok((true, Vec::new()))
            } else {
                let mut ret = m1.list_differences(&m2);
                if !same_content {
                    ret.push(("content".into(), "different".into()))
                }
                Ok((false, ret))
            }
        },
        (FileType::Dir(m1), FileType::Dir(m2)) => {
            if m1 == m2 {
                Ok((true, Vec::new()))
            } else {
                Ok((false, m1.list_differences(&m2)))
            }
        },
        // Any other case
        _ => Ok((&type1 == &type2, Vec::new())),
    }

}

/// Read `security.selinux` extended attribute
fn read_selinux_label(file: &Path) -> Result<Option<String>, Error> {
    let attribute = "security.selinux";
    if let Ok(attr) = xattr::get(file, attribute) {
        if let Some(label_bytes) = attr {
            if let Ok(label_str) = str::from_utf8(&label_bytes) {
                Ok(Some(label_str.into()))
            } else {
                Err(Error::SELinux("SELinux label is not valid UTF-8"))
            }
        } else {
            Ok(None)
        }
    } else {
        Err(Error::SELinux("Failed to read SELinux label."))
    }
}