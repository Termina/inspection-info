use bytesize::ByteSize;
use parse_size::parse_size;
use std::path::Path;

use crate::args::InspectForFileSize;
use walkdir::WalkDir;

pub fn show_file_size(options: InspectForFileSize) -> Result<(), String> {
  let min_size = parse_size(&options.min).unwrap_or_else(|_| panic!("parse file size from string: {}", &options.min));

  // Helper function to check if file matches extension filter
  let matches_extension = |path: &Path| -> bool {
    match &options.ext {
      Some(ext) => {
        if let Some(file_ext) = path.extension() {
          file_ext.to_string_lossy().to_lowercase() == ext.to_lowercase()
        } else {
          false
        }
      }
      None => true, // No filter, include all files
    }
  };

  if options.sort {
    let mut file_and_size: Vec<(u64, String)> = Vec::new();
    for entry in WalkDir::new(&options.base) {
      let entry = entry.map_err(|e| e.to_string())?;

      // Skip directories
      if entry.file_type().is_dir() {
        continue;
      }

      // Check extension filter
      if !matches_extension(entry.path()) {
        continue;
      }

      // get file size
      let the_size = entry.metadata().map_err(|e| e.to_string())?.len();
      if the_size < min_size {
        continue;
      }
      file_and_size.push((the_size, entry.path().display().to_string()));
    }
    file_and_size.sort_by(|a, b| a.0.cmp(&b.0));
    for (size, path) in file_and_size {
      println!("{} {}", ByteSize(size), path);
    }
  } else {
    for entry in WalkDir::new(&options.base) {
      let entry = entry.map_err(|e| e.to_string())?;

      // Skip directories
      if entry.file_type().is_dir() {
        continue;
      }

      // Check extension filter
      if !matches_extension(entry.path()) {
        continue;
      }

      // get file size
      let the_size = entry.metadata().map_err(|e| e.to_string())?.len();
      if the_size < min_size {
        continue;
      }
      println!("{} {}", ByteSize(the_size), entry.path().display());
    }
  }

  Ok(())
}
