use anyhow::{anyhow, Result};
use dialoguer::Editor;
use git2::Repository;
use log::debug;
use std::io::Error;
use std::ops::Deref;
use std::process::Command;

/// performs `git commit`
/// * `message` - commit message to use. If None, will prompt user to enter a commit message
///
/// returns the commit message in the Option if successfully committed. Will return None if user did not enter a commit message, or did not save
pub fn commit<'a>(message: &Option<String>) -> Result<Option<String>> {
    let commit_msg: Option<String> = match message {
        Some(message) => Some(message.deref().to_string()),
        None => Editor::new().edit("Enter a commit message").unwrap_or(None),
    };

    if commit_msg.is_none() || commit_msg.as_ref().unwrap() == "" {
        return Ok(None);
    }

    return match Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg.as_ref().unwrap())
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
            Ok(commit_msg)
        }
        Err(output) => Err(anyhow!("Error commiting changes: {}", output)),
    };
}

/// git push
pub fn push<'a>(force: bool) -> Result<()> {
    let args = {
        if force {
            vec!["push", "--force"]
        } else {
            vec!["push"]
        }
    };
    debug!("git push args: {}", args.join(" "));
    let output = Command::new("git").args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    // debug!("git push stdout: {}", &stdout);
    // debug!("git push stderr: {}", &stderr);
    println!("{}", stdout);
    println!("{}", stderr);
    return Ok(());
}

/// check if there are any staged files
///
/// return true if there are staged files. False other wise
pub fn check_staged_files() -> bool {
    let output = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .arg("--name-only")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    if stdout == "" {
        return false;
    } else {
        return true;
    }
}

/// `git add -A`
///
/// returns Some(error) if there was an error. None otherwise
pub fn add_all() -> Option<Error> {
    let err = Command::new("git").arg("add").arg("-A").spawn().err();
    debug!("adding all git changes");
    return err;
}

/// `git commit --amend --no-edit`
pub fn commit_amend_no_edit() -> Result<()> {
    //'git commit --amend --no-edit && git push --force'

    let args = vec!["commit", "--amend", "--no-edit"];
    // let output = Command::new("git").arg("push").arg("--force").output()?;
    let output = Command::new("git").args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    return Ok(());
}

/// check if there are unpushed commits
pub fn unpushed_changes() -> Result<bool> {
    // Open the repository in the current directory
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            return Err(anyhow!("Unable to open repository"));
        }
    };

    // Get the current branch name
    let branch = match repo.head() {
        Ok(reference) => {
            let shorthand = reference.shorthand();
            let shorthand = shorthand.map(|m| m.to_owned());
            if let Some(name) = shorthand {
                name
            } else {
                return Err(anyhow!("Failed to get current branch name"));
            }
        }
        Err(e) => {
            return Err(anyhow!("Failed to get current branch: {}", e));
        }
    };

    // Check if the branch is ahead of its upstream
    let branch_obj = match repo.find_branch(branch.as_str(), git2::BranchType::Local) {
        Ok(branch_obj) => branch_obj,
        Err(e) => {
            return Err(anyhow!("Failed to find branch '{}': {}", branch, e));
        }
    };

    let upstream = match branch_obj.upstream() {
        Ok(upstream) => upstream,
        Err(e) => {
            eprintln!("Branch '{}' has no upstream configured", branch);
            return Err(anyhow!("Branch '{}' has no upstream configured", branch));
        }
    };

    let ahead_behind = match repo.graph_ahead_behind(
        branch_obj.get().target().unwrap(),
        upstream.get().target().unwrap(),
    ) {
        Ok((ahead, behind)) => (ahead, behind),
        Err(e) => {
            eprintln!("Failed to get ahead/behind information: {}", e);
            return Err(anyhow!("Failed to get ahead/behind information: {}", e));
        }
    };

    if ahead_behind.0 > 0 {
        return Ok(true);
    } else {
        return Ok(false);
    }
}
