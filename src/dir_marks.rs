//! implement something like bashmarks in Rust
//! plus, each bookmark has a description, and prints in better format.
//! the return path is stored in a temp file, so that zsh can jump to the target.

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::process::exit;

use colored::Colorize;
use serde::{Deserialize, Serialize};

/// the file that stores the marks
const MARKS_CONFIG_FILE: &str = "~/.inspection-bashmarks.json";

const SHELL_FN_GG: &str = include_str!("dir_mark_gg.sh");

fn expand_home_dir(path: &str) -> String {
  let home = std::env::var("HOME").expect("get home dir");
  if path.starts_with('~') {
    path.replacen('~', &home, 1)
  } else {
    path.to_owned()
  }
}

/// tell zsh to jump to the target
const JUMP_TARGET_DATA_PATH: &str = "/tmp/inspection-bashmarks-jump-target";

#[derive(Debug, Serialize, Deserialize)]
pub struct DirMarks {
  pub marks: Vec<Bookmark>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Bookmark {
  pub kwd: String,
  pub path: String,
  pub description: String,
  pub jump_times: u64,
}

impl Bookmark {
  pub fn new(kwd: String, path: String, description: String) -> Self {
    Self {
      kwd,
      path,
      description,
      jump_times: 0,
    }
  }
}

impl DirMarks {
  /// load config from file
  pub fn load() -> Result<Self, String> {
    // check file exists first
    if !std::path::Path::new(&expand_home_dir(MARKS_CONFIG_FILE)).exists() {
      return Ok(Self { marks: Vec::new() });
    }
    let file = File::open(expand_home_dir(MARKS_CONFIG_FILE)).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let marks: Vec<Bookmark> = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
    Ok(Self { marks })
  }

  pub fn save_and_write(&self) -> Result<(), String> {
    let config_file = expand_home_dir(MARKS_CONFIG_FILE);
    let file = File::create(config_file).expect("create config file to write");
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &self.marks).expect("write json");
    Ok(())
  }

  /// if path already exists, update it, or add a new one
  pub fn add(&mut self, kwd: String, path: &str, description: String) {
    let target = self.marks.iter_mut().find(|m| m.kwd == kwd);
    if let Some(target) = target {
      path.clone_into(&mut target.path);
      description.clone_into(&mut target.description);
    } else {
      self
        .marks
        .push(Bookmark::new(kwd.to_owned(), path.to_owned(), description.to_owned()));
    }
    println!("added `{kwd}`\t{path}\t{description}")
  }

  pub fn remove(&mut self, kwd: &str) {
    let target = self.marks.iter().find(|m| m.kwd == kwd);
    if let Some(target) = target {
      println!("removed `{}`\t{}\t{}", target.kwd, target.path, target.description);
      self.marks.retain(|m| m.kwd != kwd);
    } else {
      println!("`{kwd}` not found");
    }
  }

  pub fn remove_by_path(&mut self, path: &str) {
    self.marks.retain(|m| m.path != path);
  }

  /// try jump, if not found, list possible matches
  pub fn jump(&mut self, kwd: &str) -> Result<(), String> {
    // remove file in `JUMP_TARGET_DATA_PATH`` first
    if std::path::Path::new(JUMP_TARGET_DATA_PATH).exists() {
      std::fs::remove_file(JUMP_TARGET_DATA_PATH).map_err(|e| format!("failed to remove {e}"))?;
    }

    // match by exact keyword
    let marks = self.marks.to_owned();
    let target = self.marks.iter_mut().find(|m| m.kwd == kwd);
    if let Some(target) = target {
      target.jump_times += 1;
      let file = File::create(JUMP_TARGET_DATA_PATH).expect("create file");
      let mut writer = BufWriter::new(file);
      writer.write_all(target.path.as_bytes()).expect("write to file");
      println!("{}", format!("cd {}\n", target.path).dimmed());
      self.save_and_write()?;
      return Ok(());
    }

    // match by **unique** prefix
    let mut target = None;
    let mut count_prefix: u32 = 0;
    for mark in &mut self.marks {
      if mark.kwd.starts_with(kwd) {
        count_prefix += 1;
        target = Some(mark);
      }
    }
    if count_prefix == 1 {
      let target = target.unwrap();
      target.jump_times += 1;
      let file = File::create(JUMP_TARGET_DATA_PATH).expect("create file");
      let mut writer = BufWriter::new(file);
      writer.write_all(target.path.as_bytes()).expect("write to file");
      println!("{}", format!("cd {} (by prefix)\n", target.path).dimmed());
      self.save_and_write()?;
      return Ok(());
    }

    println!("{}", "possible matches:\n".dimmed());
    let mut found = false;
    for mark in &marks {
      if mark.kwd.contains(kwd) || mark.description.contains(kwd) {
        println!(
          "{:<12} {:<48} {:<24}",
          mark.kwd.dimmed(),
          mark.path.dimmed(),
          mark.description.dimmed()
        );
        found = true;
      }
    }
    if found {
      Ok(())
    } else {
      eprintln!("{}", "No match found".red());
      Ok(())
    }
  }

  /// lookup keyword, if found, print path to stdout, otherwise exit with error
  pub fn lookup(&mut self, kwd: &str) -> Result<(), String> {
    let marks = self.marks.to_owned();
    let target = self.marks.iter_mut().find(|m| m.kwd == kwd);
    if let Some(target) = target {
      target.jump_times += 1;
      let file = File::create(JUMP_TARGET_DATA_PATH).expect("create file");
      let mut writer = BufWriter::new(file);
      writer.write_all(target.path.as_bytes()).expect("write to file");
      print!("{}", target.path);
      Ok(())
    } else {
      eprintln!("{}", "possible matches:\n".dimmed());
      let mut found = false;
      for mark in &marks {
        if mark.kwd.contains(kwd) || mark.description.contains(kwd) {
          println!(
            "{:<12} {:<48} {:<24}",
            mark.kwd.dimmed(),
            mark.path.dimmed(),
            mark.description.dimmed()
          );
          found = true;
        }
      }
      if !found {
        eprintln!("{}", "No match found".red());
      }
      exit(1); // exit with error
    }
  }

  /// list all marks
  pub fn list(&self, query: Option<&str>) {
    if self.marks.is_empty() {
      println!("No marks found");
      return;
    }
    for mark in &self.marks {
      let mut path = mark.path.clone();
      // replace home dir with ~
      let home = std::env::var("HOME").expect("get home dir");
      if path.starts_with(&home) {
        path = path.replacen(&home, "~", 1);
      }

      if let Some(query) = query {
        if !mark.kwd.contains(query) && !mark.description.contains(query) {
          continue;
        }
      }

      println!("{:<12}\t{:<36}\t{:<4}\t{:<12}", mark.kwd, path, mark.jump_times, mark.description);
    }
  }

  /// print shell function for zsh
  pub fn shell_fn() {
    println!("{SHELL_FN_GG}");
  }
}
