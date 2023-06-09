use anyhow::{anyhow, Result};
use clap::ArgAction;
use clap::{command, Parser};
use dialoguer::{Confirm, Editor};
use log::*;
use std::borrow::Cow;
use std::error::Error;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

#[derive(Parser)]
#[command(name = "gh-ac")]
#[command(author = "Shawn Yu")]
#[command(version = "1.0.0")]
#[command(about = "Fire off gh actions")]
struct Cli {
    /// git commit message
    #[arg(long, short)]
    message: Option<String>,
    /// if all changes should be committed
    #[arg(long, short)]
    all: Option<bool>,
}

use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub total_count: i64,
    pub workflow_runs: Vec<WorkflowRun>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowRun {
    pub id: i64,
    pub name: String,
    pub head_branch: String,
    pub display_title: String,
    pub status: String,
    pub conclusion: String,
    pub created_at: String,
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
    cli.all.unwrap_or(false);
    let initial_workflow_runs = get_workflow_runs();

    get_diff();
    // let commit_msg = git_commit().unwrap();
    // log::info!("commiting successful: {}", commit_msg);

    // git_push().unwrap();
}

/// get all workflow runs of a repo
fn get_workflow_runs() -> Result<Root> {
    let output = Command::new("gh")
        .arg("api")
        .arg("/repos/{owner}/{repo}/actions/runs")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(serde_json::from_str::<Root>(&stdout)?);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Command failed with error:\n{}", stderr);
        return Err(anyhow!("this command failed..."));
    }
}

/// prompts user to enter a commit message, then commits the changes to git
/// returns the commit message if successful
fn git_commit() -> Result<String> {
    let commit_msg = match Editor::new().edit("Enter a commit message").unwrap() {
        Some(msg) => msg,
        None => String::from(""),
    };

    return match Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_msg)
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

fn git_push() -> Result<()> {
    Command::new("git").arg("push").spawn()?;
    return Ok(());
}

fn get_diff() {
    // if Confirm::new()
    // .with_prompt("Do you want to continue?")
    // .default(false)
    // .interact()
    // .unwrap()
    // {
    // println!("Looks like you want to continue");
    // } else {
    // println!("nevermind then :(");
    // }
    // TODO: if nothing is staged, prompt user if they would like to continue
    let output = Command::new("git")
        .arg("diff")
        .arg("--staged")
        .arg("--name-only")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    if stdout == "" {
        print!("HHHHHa")
    }
}
