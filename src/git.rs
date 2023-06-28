use anyhow::Result;
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
