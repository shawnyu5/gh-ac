use anyhow::{anyhow, Result};
use git2::Repository;
use log::debug;
use std::process::Command;

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
pub fn check_unpushed_changes() -> Result<bool> {
    // Open the repository in the current directory
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            return Err(anyhow!("Unable to open repository: {}", e.to_string()));
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
        Err(_) => {
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
