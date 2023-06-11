mod gh;
mod git;
use crate::gh::Gh;
use clap::{command, Args, Parser};
use clap::{ArgAction, Subcommand};
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

#[derive(Args, Serialize, Deserialize, Default)]
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
    let config: ConfigArgs = confy::load("gh-ac", None).unwrap();
    let gh = Gh::new(&config.hostname.as_deref());

    match &cli.commands {
        Commands::Commit(args) => {
            let selected_workflow_name = gh.select_workflow_name();
            let initial_workflow_run = gh.get_workflow_run_by_name(&selected_workflow_name);

            if args.all {
                match git::add_all() {
                    Some(e) => panic!("{}", e),
                    None => {}
                };
            }

            if !git::check_staged_files() {
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

            gh.check_for_new_workflow_run_by_id(&initial_workflow_run.unwrap());
        }
        Commands::Force => {
            let selected_workflow_name = gh.select_workflow_name();

            if git::check_staged_files()
                && !Confirm::new()
                    .with_prompt("There are staged changes. Are you sure you want to force push?")
                    .default(false)
                    .interact()
                    .unwrap()
            {
                {
                    println!("Ok, aborting");
                    return;
                }
            }

            let initial_workflow_run = gh
                .get_workflow_run_by_name(&selected_workflow_name)
                .unwrap();

            git::commit_amend_no_edit().unwrap();
            git::push(true).unwrap();

            gh.check_for_new_workflow_run_by_id(&initial_workflow_run)
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
