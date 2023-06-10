mod gh;
mod git;
use crate::gh::get_workflow_runs;
use clap::{command, Args, Parser};
use clap::{ArgAction, Subcommand};
use dialoguer::Confirm;
use gh::WorkflowRun;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Parser)]
// TODO: fill these out in Cargo.toml and read them from there
#[command(name = "gh-ac")]
#[command(author = "Shawn Yu")]
#[command(version = "1.0.0")]
#[command(about = "Fire off gh actions")]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// commit the current changes
    Commit(CommitArgs),
    /// force push to trigger a new workflow run
    Force,
    /// set configuration values
    Config(ConfigArgs),
}

#[derive(Args)]
struct CommitArgs {
    /// add all unstaged changes before commiting
    #[arg(long, short, action = ArgAction::SetTrue)]
    all: bool,
    // /// git commit message
    // #[arg(long, short)]
    // message: Vec<String>,
}

#[derive(Args, Serialize, Deserialize)]
struct ConfigArgs {
    #[arg(long)]
    hostname: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// github custom hostname
    gh_hostname: Option<String>,
}
fn main() {
    let cli = Cli::parse();
    match &cli.commands {
        Commands::Commit(args) => {
            let initial_workflow_runs = get_workflow_runs(None).expect("workflow runs from `gh`");
            if initial_workflow_runs.total_count == 0 {
                println!("no workflow runs found, exiting");
                return;
            }

            dbg!(&args.all);
            if args.all {
                match git::add_all() {
                    Some(e) => panic!("{}", e),
                    None => {}
                };
            }
            let is_staged = git::check_staged_files();

            if !is_staged {
                println!("no staged files, exiting");
                return;
            }

            let commit_msg = git::commit(&None).unwrap();

            if commit_msg.is_none() {
                println!("no commit message was entered, exiting");
                return;
            }
            println!("commiting successful: {}", commit_msg.unwrap());

            git::push(false).unwrap();

            check_for_new_workflow(
                initial_workflow_runs
                    .workflow_runs
                    .unwrap()
                    .get(0)
                    .unwrap_or(&WorkflowRun::default()),
            )
        }
        Commands::Force => {
            let is_staged = git::check_staged_files();
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

            let initial_workflow_runs = get_workflow_runs(None).expect("workflow runs from `gh`");

            git::commit_amend_no_edit().unwrap();
            git::push(true).unwrap();

            check_for_new_workflow(
                initial_workflow_runs
                    .workflow_runs
                    .unwrap()
                    .get(0)
                    .unwrap_or(&WorkflowRun::default()),
            )
        }
        Commands::Config(manage_config) => {
            // hostname is a required field, so we can unwrap it here
            let config = ConfigArgs {
                hostname: manage_config.hostname.clone(),
            };
            confy::store("gh-ac", None, config).unwrap();
            println!("config saved");
        }
    }
}

/// check for new workflow runs
///
/// * `old_workflow_run`: the latest workflow run in the repo
fn check_for_new_workflow(old_workflow_run: &WorkflowRun) {
    loop {
        let current_workflow_runs = get_workflow_runs(None).expect("workflow runs from gh");

        let same = {
            old_workflow_run
                == current_workflow_runs
                    .clone()
                    .workflow_runs
                    .unwrap()
                    .get(0)
                    .unwrap()
        };

        println!("new workflow as not started");
        if same {
            println!("waiting for 5 seconds");
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        }
        println!(
            "{}",
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
