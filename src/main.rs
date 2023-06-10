mod gh;
mod git;
use crate::gh::get_workflow_runs;
use clap::Subcommand;
use clap::{command, Args, Parser};
use dialoguer::Confirm;
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
    /// set configuration values
    Config(ManageConfig), // Config { set: Option<String> },
}

#[derive(Args, Serialize, Deserialize)]
struct ManageConfig {
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
        Commands::Commit => {
            let initial_workflow_runs = get_workflow_runs(None).expect("workflow runs from `gh`");
            if initial_workflow_runs.total_count == 0 {
                println!("no workflow runs found, exiting");
                return;
            }

            let (is_staged, _) = git::check_staged_files();

            if !is_staged {
                println!("no staged files, exiting");
                return;
            }

            let commit_msg = git::commit().unwrap();

            if commit_msg.is_none() {
                println!("no commit message was entered, exiting");
                return;
            }
            println!("commiting successful: {}", commit_msg.unwrap());

            git::push(false).unwrap();

            loop {
                let current_workflow_runs = get_workflow_runs(None).expect("workflow runs from gh");
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
            let (is_staged, _) = git::check_staged_files();
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

            loop {
                let current_workflow_runs = get_workflow_runs(None).expect("workflow runs from gh");

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
        Commands::Config(manage_config) => {
            // hostname is a required field, so we can unwrap it here
            let config = ManageConfig {
                hostname: manage_config.hostname.clone(),
            };
            confy::store("gh-ac", None, config).unwrap();
            println!("config saved");
        }
    }
}
