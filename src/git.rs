use git2::{BranchType, Repository};
use std::io::{self, Write};
use std::process::Command;

/// detect main or master branch first, get current branch,
/// check for dirty state and handle stashing if needed,
/// then fetch origin/main --prune, switch to main, pull latest changes
/// finally delete that branch and restore stashed changes if any
pub fn finish_branch() -> Result<(), Box<dyn std::error::Error>> {
  let repo = Repository::open(".")?;

  println!("🔍 Starting branch finish process...");

  // Get the current branch
  let head = repo.head()?;
  let current_branch_name = if let Some(name) = head.shorthand() {
    name.to_string()
  } else {
    return Err("Not on a branch".into());
  };

  println!("📍 Current branch: {current_branch_name}");

  // Check if we're already on main or master
  if current_branch_name == "main" || current_branch_name == "master" {
    println!("✅ Already on {current_branch_name} branch, nothing to do");
    return Ok(());
  }

  // Check for dirty working directory
  let has_stashed_changes = check_and_handle_dirty_state()?;

  // Detect the main branch (main or master)
  let main_branch = detect_main_branch(&repo)?;
  println!("🎯 Detected main branch: {main_branch}");

  // Switch to main branch
  println!("🔄 Switching to {main_branch} branch...");
  checkout_branch(&repo, &main_branch)?;
  println!("✅ Successfully switched to {main_branch} branch");

  // First fetch to get the latest remote history
  println!("📡 Fetching latest changes from remote...");
  fetch_remote()?;

  // Check if the current branch has been merged into main
  println!("🔍 Checking if branch '{current_branch_name}' has been merged into '{main_branch}'...");
  if !is_branch_merged(&current_branch_name, &main_branch)? {
    println!("⚠️  Warning: Branch '{current_branch_name}' has not been merged into '{main_branch}'!");
    println!("   This means the branch contains commits that are not in the main branch.");
    println!("   Please merge or rebase your branch first before using this command.");
    println!("   Aborting to prevent data loss.");
    return Err("Branch not merged - cannot safely delete".into());
  }
  println!("✅ Branch '{current_branch_name}' has been merged into '{main_branch}'");

  // Pull latest changes on main branch
  println!("📥 Pulling latest changes on {main_branch} branch...");
  pull_main_branch(&main_branch)?;

  // Delete the feature branch (no confirmation needed)
  println!("🗑️  Deleting branch '{current_branch_name}'...");
  delete_branch(&repo, &current_branch_name)?;
  println!("✅ Successfully deleted branch '{current_branch_name}'");

  // Restore stashed changes if any
  if has_stashed_changes {
    println!("🔄 Restoring previously stashed changes...");
    restore_stashed_changes()?;
  }

  println!("🎉 Finished! You are now on the {main_branch} branch");
  Ok(())
}

pub fn open_remote_repository() -> Result<(), String> {
  use std::fs;
  use std::process::Command;
  
  // 读取 .git/config 文件
  let config_content = fs::read_to_string(".git/config")
    .map_err(|e| format!("Failed to read .git/config: {e}"))?;
  
  // 查找 remote "origin" 部分的 url
  let mut in_origin_section = false;
  let mut remote_url = None;
  
  for line in config_content.lines() {
    let line = line.trim();
    
    if line == "[remote \"origin\"]" {
      in_origin_section = true;
      continue;
    }
    
    if line.starts_with('[') && line != "[remote \"origin\"]" {
      in_origin_section = false;
      continue;
    }
    
    if in_origin_section && line.starts_with("url = ") {
      let url = line.strip_prefix("url = ").unwrap();
      remote_url = Some(url.to_string());
      break;
    }
  }
  
  let url = remote_url.ok_or("No remote origin URL found in .git/config")?;
  
  // 转换 Git URL 为 HTTP URL
  let web_url = if url.starts_with("git@") {
    // SSH format: git@github.com:user/repo.git -> https://github.com/user/repo
    let without_git = url.strip_prefix("git@").unwrap();
    let parts: Vec<&str> = without_git.split(':').collect();
    if parts.len() == 2 {
      let host = parts[0];
      let path = parts[1].strip_suffix(".git").unwrap_or(parts[1]);
      format!("https://{host}/{path}")
    } else {
      return Err("Invalid SSH Git URL format".to_string());
    }
  } else if url.starts_with("https://") {
    // HTTPS format: already web-compatible, just remove .git suffix if present
    url.strip_suffix(".git").unwrap_or(&url).to_string()
  } else {
    return Err("Unsupported Git URL format".to_string());
  };
  
  println!("🌐 Opening remote repository: {web_url}");
  
  // 使用系统默认浏览器打开 URL
  let result = Command::new("open")
    .arg(&web_url)
    .output()
    .map_err(|e| format!("Failed to open browser: {e}"))?;
  
  if !result.status.success() {
    let stderr = String::from_utf8_lossy(&result.stderr);
    return Err(format!("Failed to open URL: {stderr}"));
  }
  
  println!("✅ Successfully opened remote repository in browser");
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

fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let branch = repo.find_branch(branch_name, BranchType::Local)?;
  let branch_ref = branch.get();
  let commit = branch_ref.peel_to_commit()?;

  repo.checkout_tree(commit.as_object(), None)?;
  repo.set_head(&format!("refs/heads/{branch_name}"))?;

  Ok(())
}

fn delete_branch(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
  let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
  branch.delete()?;
  Ok(())
}

fn check_and_handle_dirty_state() -> Result<bool, Box<dyn std::error::Error>> {
  // Check if there are uncommitted changes
  let status = Command::new("git").args(["status", "--porcelain"]).output()?;

  if status.stdout.is_empty() {
    println!("✅ Working directory is clean");
    return Ok(false);
  }

  // Show what changes would be stashed
  println!("⚠️  Detected uncommitted changes in working directory:");
  let status_output = String::from_utf8_lossy(&status.stdout);
  for line in status_output.lines() {
    println!("   {line}");
  }

  // Ask user for confirmation
  print!("📦 Do you want to stash these changes before proceeding? (y/N): ");
  io::stdout().flush()?;

  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
    println!("📦 Stashing uncommitted changes...");

    // Create a stash with a descriptive message
    let stash_message = format!(
      "Auto-stash before branch finish at {}",
      std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs()
    );

    let status = Command::new("git").args(["stash", "push", "-m", &stash_message]).status()?;

    if !status.success() {
      return Err("Failed to stash changes".into());
    }

    println!("✅ Successfully stashed changes with message: '{stash_message}'");
    Ok(true)
  } else {
    Err("Cannot proceed with uncommitted changes. Please commit or stash them manually.".into())
  }
}

fn restore_stashed_changes() -> Result<(), Box<dyn std::error::Error>> {
  // Apply the most recent stash
  let status = Command::new("git").args(["stash", "pop"]).output()?;

  if !status.status.success() {
    let error_msg = String::from_utf8_lossy(&status.stderr);
    if error_msg.contains("No stash entries found") {
      println!("ℹ️  No stash entries to restore");
      return Ok(());
    } else {
      println!("⚠️  Warning: Failed to automatically restore stashed changes:");
      println!("   {}", error_msg.trim());
      println!("   You can manually restore them later with: git stash pop");
      return Ok(()); // Don't fail the entire operation
    }
  }

  println!("✅ Successfully restored stashed changes");

  // Show what was restored
  let restored_output = String::from_utf8_lossy(&status.stdout);
  if !restored_output.trim().is_empty() {
    println!("📋 Restored changes:");
    for line in restored_output.lines() {
      if !line.trim().is_empty() {
        println!("   {line}");
      }
    }
  }

  Ok(())
}

fn fetch_remote() -> Result<(), Box<dyn std::error::Error>> {
  println!("📡 Fetching from origin with --prune...");
  let status = Command::new("git").args(["fetch", "origin", "--prune"]).status()?;

  if !status.success() {
    return Err(format!("Failed to fetch from origin with exit code: {}", status.code().unwrap_or(-1)).into());
  }

  println!("✅ Successfully fetched from origin");
  Ok(())
}

fn is_branch_merged(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  // Use git merge-base to check if the branch has been merged
  // If the merge-base of the branch and main is the same as the branch's HEAD,
  // then the branch has been fully merged into main
  let branch_head = Command::new("git").args(["rev-parse", branch_name]).output()?;

  if !branch_head.status.success() {
    return Err(format!("Failed to get HEAD of branch '{branch_name}'").into());
  }

  let branch_head_hash = String::from_utf8_lossy(&branch_head.stdout).trim().to_string();

  let merge_base = Command::new("git").args(["merge-base", branch_name, main_branch]).output()?;

  if !merge_base.status.success() {
    return Err(format!("Failed to find merge-base between '{branch_name}' and '{main_branch}'").into());
  }

  let merge_base_hash = String::from_utf8_lossy(&merge_base.stdout).trim().to_string();

  // If the branch HEAD is the same as the merge-base, the branch is fully merged
  Ok(branch_head_hash == merge_base_hash)
}

fn pull_main_branch(main_branch: &str) -> Result<(), Box<dyn std::error::Error>> {
  println!("🔄 Merging origin/{main_branch}...");
  let status = Command::new("git").args(["merge", &format!("origin/{main_branch}")]).status()?;

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

  println!("✅ Successfully updated {main_branch} branch");
  Ok(())
}
