use anyhow::Result;
use log::debug;
use std::process::Command;

pub struct Git {
    quiet: bool,
}

impl Git {
    /// construct a new Git instance
    ///
    /// * `quiet`: if true, then git commands will be run with the `-q` flag
    pub fn new(quiet: bool) -> Git {
        Git { quiet }
    }
    fn construct_command(&self, args: &Vec<&str>) -> Command {
        if self.quiet {
            let mut cmd = Command::new("git");
            cmd.args(args).arg("-q");
            return cmd;
        } else {
            let mut cmd = Command::new("git");
            cmd.args(args);
            return cmd;
        }
    }
    /// git push
    pub fn push<'a>(&self, force: bool) -> Result<()> {
        let args = {
            if force {
                vec!["push", "--force"]
            } else {
                vec!["push"]
            }
        };
        debug!("git push args: {}", args.join(" "));
        let output = self.construct_command(&args).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("{}", stdout);
        println!("{}", stderr);
        return Ok(());
    }

    /// check if there are any staged files
    ///
    /// return true if there are staged files. False other wise
    pub fn check_staged_files(&self) -> bool {
        let output = self
            .construct_command(&vec!["diff", "--staged", "--name-only"])
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
    pub fn commit_amend_no_edit(&self) -> Result<()> {
        let output = self
            .construct_command(&vec!["commit", "--amend", "--no-edit"])
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
        return Ok(());
    }
}
