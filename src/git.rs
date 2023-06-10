use anyhow::{anyhow, Result};
use dialoguer::Editor;
use std::io::Error;
use std::process::Command;

/// prompts user to enter a commit message, then commits the changes to git
/// returns the commit message in the Option if successful. Commit message will return None if user did not enter a commit message, or did not save
pub fn commit() -> Result<Option<String>> {
    let commit_msg: Option<String> = Editor::new().edit("Enter a commit message").unwrap_or(None);

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
/// return a tuple of (bool, String) where the bool is true if there are staged files, containing the staged file names. False other wise
pub fn check_staged_files() -> (bool, String) {
    let output = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .arg("--name-only")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    if stdout == "" {
        return (false, String::from(""));
    } else {
        return (true, stdout.to_string());
    }
}

/// `git add -A`
/// returns None if successful, Some(Error) if failed
pub fn add_all() -> Option<Error> {
    let output = Command::new("git").arg("add").arg("-A").spawn();
    return output.err();
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
