use anyhow::{anyhow, Result};
use dialoguer::Editor;
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
pub fn push(force: bool) -> Result<()> {
    let args = {
        if force {
            vec!["push", "--force"]
        } else {
            vec!["push"]
        }
    };
    let output = Command::new("git").args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
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
