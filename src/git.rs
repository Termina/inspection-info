use git2::{BranchType, Repository};
use std::io::{self, Write};
use std::process::Command;

/// detect main or master branch first, get current branch,
/// then fetch origin/main --prune, switch to main, pull latest changes
/// finally delete that branch
pub fn finish_branch() -> Result<(), Box<dyn std::error::Error>> {
  let repo = Repository::open(".")?;

  // Get the current branch
  let head = repo.head()?;
  let current_branch_name = if let Some(name) = head.shorthand() {
    name.to_string()
  } else {
    return Err("Not on a branch".into());
  };

  // Check if we're already on main or master
  if current_branch_name == "main" || current_branch_name == "master" {
    println!("Already on {} branch, nothing to do", current_branch_name);
    return Ok(());
  }

  // Detect the main branch (main or master)
  let main_branch = detect_main_branch(&repo)?;
  println!("Detected main branch: {}", main_branch);

  // Switch to main branch
  println!("Switching to {} branch...", main_branch);
  checkout_branch(&repo, &main_branch)?;

  // Fetch and pull latest changes on main branch in one operation
  println!("Fetching and pulling latest changes on {} branch...", main_branch);
  fetch_and_pull(&repo, &main_branch)?;

  // Delete the feature branch
  print!("Delete branch '{}'? (y/N): ", current_branch_name);
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
    delete_branch(&repo, &current_branch_name)?;
    println!("Deleted branch '{}'", current_branch_name);
  } else {
    println!("Branch '{}' was not deleted", current_branch_name);
  }

  println!("Finished! You are now on the {} branch", main_branch);
  Ok(())
}

fn detect_main_branch(repo: &Repository) -> Result<String, Box<dyn std::error::Error>> {
  // First try to find main branch
  if repo.find_branch("main", BranchType::Local).is_ok() {
    return Ok("main".to_string());
  }

  // Then try master branch
  if repo.find_branch("master", BranchType::Local).is_ok() {
    return Ok("master".to_string());
  }

  // Try to find remote main/master branches
  let branches = repo.branches(Some(BranchType::Remote))?;
  for branch_result in branches {
    let (branch, _) = branch_result?;
    if let Some(name) = branch.name()? {
      if name.ends_with("/main") {
        return Ok("main".to_string());
      } else if name.ends_with("/master") {
        return Ok("master".to_string());
      }
    }
  }

  Err("Could not detect main or master branch".into())
}

fn fetch_and_pull(_repo: &Repository, main_branch: &str) -> Result<(), Box<dyn std::error::Error>> {
  // Check if there are uncommitted changes first
  let status = Command::new("git").args(["status", "--porcelain"]).output()?;
  if !status.stdout.is_empty() {
    return Err("There are uncommitted changes. Please commit or stash them first.".into());
  }

  // Fetch all branches with prune and then merge the remote tracking branch
  // This is more efficient than separate fetch + pull commands
  let status = Command::new("git").args(["fetch", "origin", "--prune"]).status()?;

  if !status.success() {
    return Err(format!("Failed to fetch from origin with exit code: {}", status.code().unwrap_or(-1)).into());
  }

  // Now merge the remote tracking branch (equivalent to pull but without redundant fetch)
  let status = Command::new("git").args(["merge", &format!("origin/{}", main_branch)]).status()?;

  if !status.success() {
    return Err(
      format!(
        "Failed to merge origin/{} with exit code: {}",
        main_branch,
        status.code().unwrap_or(-1)
      )
      .into(),
    );
  }

  println!("Successfully fetched and updated {} branch", main_branch);
  Ok(())
}

fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let branch = repo.find_branch(branch_name, BranchType::Local)?;
  let branch_ref = branch.get();
  let commit = branch_ref.peel_to_commit()?;

  repo.checkout_tree(commit.as_object(), None)?;
  repo.set_head(&format!("refs/heads/{}", branch_name))?;

  Ok(())
}

fn delete_branch(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
  branch.delete()?;
  Ok(())
}
