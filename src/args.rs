use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Top-level command.
pub struct TopLevelInspection {
  #[argh(subcommand)]
  pub subcommand: InspectionCommand,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum InspectionCommand {
  InIp(InspectForIp),
  CopyFile(InspectForCopyFile),
  ShowMemory(InspectForMemory),
  ShowProcesses(InspectForProcesses),
  ShowWorkingDirectory(InspectForWorkingDirectory),
  ListFileSize(InspectForFileSize),
  DirMark(InspectForDirMark),
  FinishBranch(InspectForFinishBranch),
}

/// command for inspecting IP addresses.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ip")]
pub struct InspectForIp {
  #[argh(switch, short = 'd')]
  /// switch on verbose mode.
  pub detailed: bool,
}

/// command for copying files.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "cpfile")]
pub struct InspectForCopyFile {
  /// source file
  #[argh(positional)]
  pub file: String,
}

/// command for showing memory.
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "mem")]
pub struct InspectForMemory {}

/// command for displaying processes
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ps")]
pub struct InspectForProcesses {}

/// command for displaying working directory
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "wd")]
pub struct InspectForWorkingDirectory {
  /// relative path
  #[argh(positional)]
  pub relative: Option<String>,
}

/// command for displaying file size
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "large")]
pub struct InspectForFileSize {
  /// minimum size
  #[argh(option, default = "String::from(\"1m\")")]
  pub min: String,
  /// base path
  #[argh(positional, default = "String::from(\".\")")]
  pub base: String,
  /// show sorted result
  #[argh(switch, short = 's')]
  pub sort: bool,
}

/// command for directory marks
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "dir")]
pub struct InspectForDirMark {
  #[argh(subcommand)]
  pub subcommand: DirMarkCommand,
}

// InspectForFinishBranch
/// command for finishing a branch
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "finbr")]
pub struct InspectForFinishBranch {}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum DirMarkCommand {
  Add(InspectForDirMarkAdd),
  Remove(InspectForDirMarkRemove),
  Search(InspectForDirMarkSearch),
  Jump(InspectForDirMarkJump),
  Lookup(InspectForDirMarkLookup),
  ShellFn(InspectForDirMarkShellFn),
}

/// command for adding a directory mark
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "add")]
pub struct InspectForDirMarkAdd {
  /// keyword
  #[argh(positional)]
  pub kwd: String,
  /// description
  #[argh(option)]
  pub desc: Option<String>,
}

/// command for removing a directory mark
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "rm")]
pub struct InspectForDirMarkRemove {
  /// keyword
  #[argh(positional)]
  pub kwd: String,
  /// remove by path
  #[argh(switch, short = 'p')]
  pub by_path: bool,
}

/// command for searching a directory mark
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ls")]
pub struct InspectForDirMarkSearch {
  /// query
  #[argh(positional)]
  pub query: Option<String>,
}

/// command for jumping to a directory mark
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "jump")]
pub struct InspectForDirMarkJump {
  /// keyword
  #[argh(positional)]
  pub kwd: String,
}

/// command for looking up a directory mark
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "lookup")]
pub struct InspectForDirMarkLookup {
  /// keyword
  #[argh(positional)]
  pub kwd: String,
}

/// command for shell function
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gg")]
pub struct InspectForDirMarkShellFn {}
