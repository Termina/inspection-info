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

  // First fetch to get the latest remote history BEFORE checking merge status
  println!("📡 Fetching latest changes from remote...");
  if let Err(e) = fetch_remote() {
    println!("⚠️  Warning: Failed to fetch from remote: {e}");
    println!("   Continuing with local state, but results may not be accurate");
  }

  // Check if the current branch has been merged into main BEFORE switching branches
  println!("🔍 Checking if branch '{current_branch_name}' has been merged into '{main_branch}'...");

  let is_merged = match is_branch_merged(&current_branch_name, &main_branch) {
    Ok(merged) => merged,
    Err(e) => {
      println!("❌ Failed to check merge status: {e}");
      println!("   Staying on current branch '{current_branch_name}' for safety");
      return Err("Failed to verify merge status - aborting to prevent data loss".into());
    }
  };

  if !is_merged {
    println!("⚠️  Warning: Branch '{current_branch_name}' has not been merged into '{main_branch}'!");
    println!("   This means the branch contains commits that are not in the main branch.");
    println!("   Please merge or rebase your branch first before using this command.");
    println!("   Staying on current branch '{current_branch_name}' for safety");
    return Err("Branch not merged - cannot safely delete".into());
  }

  println!("✅ Branch '{current_branch_name}' has been merged into '{main_branch}'");

  // Now it's safe to switch to main branch
  println!("🔄 Switching to {main_branch} branch...");
  if let Err(e) = checkout_branch(&repo, &main_branch) {
    println!("❌ Failed to switch to {main_branch} branch: {e}");
    println!("   Staying on current branch '{current_branch_name}' for safety");
    return Err(format!("Failed to switch to {main_branch} branch").into());
  }
  println!("✅ Successfully switched to {main_branch} branch");

  // Pull latest changes on main branch
  println!("📥 Pulling latest changes on {main_branch} branch...");
  if let Err(e) = pull_main_branch(&main_branch) {
    println!("⚠️  Warning: Failed to pull latest changes: {e}");
    println!("   Continuing with branch deletion...");
  }

  // Delete the feature branch (no confirmation needed)
  println!("🗑️  Deleting branch '{current_branch_name}'...");
  if let Err(e) = delete_branch(&repo, &current_branch_name) {
    println!("❌ Failed to delete branch '{current_branch_name}': {e}");
    println!("   You may need to delete it manually with: git branch -d {current_branch_name}");
  } else {
    println!("✅ Successfully deleted branch '{current_branch_name}'");
  }

  // Restore stashed changes if any
  if has_stashed_changes {
    println!("🔄 Restoring previously stashed changes...");
    if let Err(e) = restore_stashed_changes() {
      println!("⚠️  Warning: Failed to restore stashed changes: {e}");
      println!("   You can manually restore them later with: git stash pop");
    }
  }

  println!("🎉 Finished! You are now on the {main_branch} branch");
  Ok(())
}

pub fn open_remote_repository() -> Result<(), String> {
  use std::fs;
  use std::process::Command;

  // 获取当前分支名
  let repo = Repository::open(".").map_err(|e| format!("Failed to open repository: {e}"))?;

  let head = repo.head().map_err(|e| format!("Failed to get HEAD: {e}"))?;

  let current_branch = if let Some(branch_name) = head.shorthand() {
    branch_name.to_string()
  } else {
    "HEAD".to_string()
  };

  // 读取 .git/config 文件
  let config_content = fs::read_to_string(".git/config").map_err(|e| format!("Failed to read .git/config: {e}"))?;

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
  let mut web_url = if url.starts_with("git@") {
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

  // 如果是 GitHub 仓库，添加分支信息
  if web_url.contains("github.com") && current_branch != "HEAD" {
    web_url = format!("{web_url}/tree/{current_branch}");
  }

  println!("🌐 Opening remote repository: {web_url} (branch: {current_branch})");

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

/// Helper function to execute git command with remote-first, local-fallback strategy
fn run_git_with_fallback(remote_args: &[&str], local_args: &[&str]) -> Result<std::process::Output, Box<dyn std::error::Error>> {
  let remote_result = Command::new("git").args(remote_args).output();

  match remote_result {
    Ok(output) if output.status.success() => Ok(output),
    _ => {
      let local_result = Command::new("git").args(local_args).output()?;
      Ok(local_result)
    }
  }
}

/// Check if a branch appears in the merged branches list
fn check_branch_in_merged_list(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  println!("🔍 Method 1: Checking with 'git branch --merged'...");

  let output = run_git_with_fallback(
    &["branch", "--merged", &format!("origin/{main_branch}")],
    &["branch", "--merged", main_branch],
  )?;

  if output.status.success() {
    let merged_output = String::from_utf8_lossy(&output.stdout);
    for line in merged_output.lines() {
      let branch_line = line.trim().trim_start_matches("* ").trim();
      if branch_line == branch_name {
        println!("✅ Method 1: Branch found in merged branches list");
        return Ok(true);
      }
    }
  }

  Ok(false)
}

/// Check if all commits in branch exist in main branch
fn check_unmerged_commits(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  println!("🔍 Method 2: Checking if all branch commits exist in main branch...");

  let output = run_git_with_fallback(
    &["rev-list", &format!("origin/{main_branch}..{branch_name}")],
    &["rev-list", &format!("{main_branch}..{branch_name}")],
  )?;

  if output.status.success() {
    let unmerged_output_binding = String::from_utf8_lossy(&output.stdout);
    let unmerged_output = unmerged_output_binding.trim();
    if unmerged_output.is_empty() {
      println!("✅ Method 2: No unmerged commits found - branch is fully merged");
      return Ok(true);
    }
  }

  Ok(false)
}

/// Check remote tracking branches
fn check_remote_tracking_branches(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  println!("🔍 Method 3: Checking remote tracking branches...");

  let remote_branches = Command::new("git")
    .args(["branch", "-r", "--merged", &format!("origin/{main_branch}")])
    .output()?;

  if remote_branches.status.success() {
    let remote_output = String::from_utf8_lossy(&remote_branches.stdout);
    let remote_branch_name = format!("origin/{branch_name}");

    for line in remote_output.lines() {
      let branch_line = line.trim();
      if branch_line == remote_branch_name {
        println!("✅ Method 3: Remote branch {remote_branch_name} found in merged list");
        return Ok(true);
      }
    }
  }

  Ok(false)
}

/// Check if branch is fast-forward merged using merge-base
fn check_fast_forward_merge(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  println!("🔍 Method 4: Checking with merge-base (fast-forward detection)...");

  let branch_head = Command::new("git").args(["rev-parse", branch_name]).output()?;
  if !branch_head.status.success() {
    return Ok(false);
  }

  let branch_head_hash = String::from_utf8_lossy(&branch_head.stdout).trim().to_string();

  let merge_base = run_git_with_fallback(
    &["merge-base", branch_name, &format!("origin/{main_branch}")],
    &["merge-base", branch_name, main_branch],
  )?;

  if merge_base.status.success() {
    let merge_base_hash = String::from_utf8_lossy(&merge_base.stdout).trim().to_string();
    if branch_head_hash == merge_base_hash {
      println!("✅ Method 4: Branch is fast-forward merged");
      return Ok(true);
    }
  }

  Ok(false)
}

fn is_branch_merged(branch_name: &str, main_branch: &str) -> Result<bool, Box<dyn std::error::Error>> {
  // Try multiple methods to detect if branch is merged
  let methods = [
    check_branch_in_merged_list,
    check_unmerged_commits,
    check_remote_tracking_branches,
    check_fast_forward_merge,
  ];

  for method in &methods {
    if method(branch_name, main_branch)? {
      return Ok(true);
    }
  }

  // If all methods failed, show debug info
  println!("❌ All merge detection methods indicate the branch has not been merged");

  // Get unmerged commits for debugging
  let unmerged_commits = run_git_with_fallback(
    &["rev-list", &format!("origin/{main_branch}..{branch_name}")],
    &["rev-list", &format!("{main_branch}..{branch_name}")],
  );

  if let Ok(output) = unmerged_commits {
    if output.status.success() {
      let unmerged_output_binding = String::from_utf8_lossy(&output.stdout);
      let unmerged_output = unmerged_output_binding.trim();
      let commit_hashes: Vec<&str> = unmerged_output.lines().collect();

      if !commit_hashes.is_empty() {
        println!("   Unmerged commits found: {}", commit_hashes.len());
        println!("   Some unmerged commits:");

        for (i, commit_hash) in commit_hashes.iter().take(3).enumerate() {
          if commit_hash.trim().is_empty() {
            continue;
          }

          if let Ok(commit_msg) = Command::new("git")
            .args(["log", "--format=%s", "-n", "1", commit_hash.trim()])
            .output()
          {
            let msg_binding = String::from_utf8_lossy(&commit_msg.stdout);
            let msg = msg_binding.trim();
            println!("     {}. {} - {}", i + 1, &commit_hash.trim()[..8], msg);
          }
        }

        if commit_hashes.len() > 3 {
          println!("     ... and {} more commits", commit_hashes.len() - 3);
        }
      }
    }
  }

  Ok(false)
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
