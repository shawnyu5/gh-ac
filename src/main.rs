use anyhow::{anyhow, Result};
use clap::{command, Parser};
use clap::{ArgAction, Subcommand};
use dialoguer::{Confirm, Editor};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::io::Error;
use std::process::Command;

#[derive(Parser)]
// TODO: fill these out in Cargo.toml and read them from there
#[command(name = "gh-ac")]
#[command(author = "Shawn Yu")]
#[command(version = "1.0.0")]
#[command(about = "Fire off gh actions")]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
    // /// git commit message
    // #[arg(long, short)]
    // message: Option<String>,
    // /// if all changes should be committed
    // #[arg(long, short)]
    // all: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    /// commit the current changes
    Commit,
    /// force push the current changes
    Force,
}

/// workflow runs of a repo from gh api
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowRuns {
    pub total_count: i64,
    pub workflow_runs: Option<Vec<WorkflowRun>>,
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: i64,
    pub name: String,
    pub head_branch: String,
    pub display_title: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub created_at: String,
    pub html_url: String,
    pub run_number: i32,
    pub updated_at: String,
    pub run_started_at: String,
    pub jobs_url: String,
    pub logs_url: String,
    pub check_suite_url: String,
    pub artifacts_url: String,
    pub workflow_url: String,
}

fn main() {
    let cli = Cli::parse();
    match &cli.commands {
        Commands::Commit => {
            let initial_workflow_runs = get_workflow_runs().expect("workflow runs from `gh`");
            if initial_workflow_runs.total_count == 0 {
                println!("no workflow runs found, exiting");
                return;
            }

            let (is_staged, _) = check_staged_files();

            if !is_staged {
                println!("no staged files, exiting");
                return;
            }

            let commit_msg = git_commit().unwrap();

            if commit_msg.is_none() {
                println!("no commit message was entered, exiting");
                return;
            }
            println!("commiting successful: {}", commit_msg.unwrap());

            git_push(false).unwrap();

            loop {
                let current_workflow_runs = get_workflow_runs().expect("workflow runs from gh");
                // dbg!(&initial_workflow_runs);
                // dbg!(&current_workflow_runs);

                let same = {
                    initial_workflow_runs
                        .clone()
                        .workflow_runs
                        .unwrap()
                        .get(0)
                        .unwrap()
                        == current_workflow_runs
                            .clone()
                            .workflow_runs
                            .unwrap()
                            .get(0)
                            .unwrap()
                };

                dbg!(&same);
                if same {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }
                println!(
                    "Workflow run url: {}",
                    current_workflow_runs
                        .clone()
                        .workflow_runs
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .html_url
                );
                break;
            }
        }
        Commands::Force => {
            let (is_staged, _) = check_staged_files();
            if is_staged {
                if !Confirm::new()
                    .with_prompt("There are staged changes. Are you sure you want to force push?")
                    .default(false)
                    .interact()
                    .unwrap()
                {
                    println!("Ok, aborting");
                    return;
                }
            }

            let initial_workflow_runs = get_workflow_runs().expect("workflow runs from `gh`");

            git_commit_amend_no_edit().unwrap();
            git_push(true).unwrap();

            loop {
                let current_workflow_runs = get_workflow_runs().expect("workflow runs from gh");
                // dbg!(&initial_workflow_runs);
                // dbg!(&current_workflow_runs);

                let same = {
                    initial_workflow_runs
                        .clone()
                        .workflow_runs
                        .unwrap()
                        .get(0)
                        .unwrap()
                        == current_workflow_runs
                            .clone()
                            .workflow_runs
                            .unwrap()
                            .get(0)
                            .unwrap()
                };

                dbg!(&same);
                if same {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    continue;
                }
                println!(
                    "Workflow run url: {}",
                    current_workflow_runs
                        .clone()
                        .workflow_runs
                        .unwrap()
                        .get(0)
                        .unwrap()
                        .html_url
                );
                break;
            }
        }
    }
}

/// get all workflow runs of a repo
fn get_workflow_runs() -> Result<WorkflowRuns> {
    let output = Command::new("gh")
        .arg("api")
        .arg("/repos/{owner}/{repo}/actions/runs")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(serde_json::from_str::<WorkflowRuns>(&stdout)?);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed with error:\n{}", stderr);
        return Err(anyhow!("this command failed..."));
    }
}

/// prompts user to enter a commit message, then commits the changes to git
/// returns the commit message in the Option if successful. Commit message will return None if user did not enter a commit message, or did not save
fn git_commit() -> Result<Option<String>> {
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
fn git_push(force: bool) -> Result<()> {
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
fn check_staged_files() -> (bool, String) {
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
fn git_add_all() -> Option<Error> {
    let output = Command::new("git").arg("add").arg("-A").spawn();
    return output.err();
}

/// `git commit --amend --no-edit`
fn git_commit_amend_no_edit() -> Result<()> {
    //'git commit --amend --no-edit && git push --force'

    let args = vec!["commit", "--amend", "--no-edit"];
    // let output = Command::new("git").arg("push").arg("--force").output()?;
    let output = Command::new("git").args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    return Ok(());
}
