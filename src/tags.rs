use crate::args::InspectForTags;
use std::process::Command;

#[derive(Clone)]
struct TagInfo {
  date: String,
  name: String,
  subject: String,
}

pub fn show_tags(options: &InspectForTags) -> Result<(), String> {
  let output = Command::new("git")
    .args([
      "for-each-ref",
      "--sort=creatordate", // Sort by date ascending
      "--format=%(creatordate:short)\t%(refname:short)\t%(subject)",
      "refs/tags",
    ])
    .output()
    .map_err(|e| e.to_string())?;

  if !output.status.success() {
    return Err(String::from_utf8_lossy(&output.stderr).to_string());
  }

  let output_str = String::from_utf8_lossy(&output.stdout);
  if output_str.is_empty() {
    println!("No tags found.");
    return Ok(());
  }

  let all_tags: Vec<TagInfo> = output_str
    .lines()
    .filter_map(|line| {
      let parts: Vec<&str> = line.splitn(3, '\t').collect();
      if parts.len() == 3 {
        Some(TagInfo {
          date: parts[0].to_string(),
          name: parts[1].to_string(),
          subject: parts[2].to_string(),
        })
      } else {
        None
      }
    })
    .collect();

  let total_tags = all_tags.len();
  let tags_to_display = if options.all {
    all_tags.clone()
  } else {
    let tags: Vec<TagInfo> = all_tags.iter().rev().take(10).cloned().collect();
    tags.into_iter().rev().collect()
  };

  if tags_to_display.is_empty() {
    println!("No tags to display.");
    return Ok(());
  }

  if !options.all && total_tags > tags_to_display.len() {
    if let (Some(first_tag), Some(last_tag)) = (all_tags.first(), all_tags.last()) {
      println!(
        "Showing the last {} of {} tags (from {} to {}). Use --all to see all.",
        tags_to_display.len(),
        total_tags,
        first_tag.date,
        last_tag.date
      );
      println!(); // Add a blank line for separation
    }
  }

  let mut max_tag_width = 0;
  for tag in &tags_to_display {
    if tag.name.len() > max_tag_width {
      max_tag_width = tag.name.len();
    }
  }

  for tag in tags_to_display {
    println!("{}  {:<width$}  {}", tag.date, tag.name, tag.subject, width = max_tag_width);
  }

  Ok(())
}
