mod gh;
mod git;
use crate::gh::Gh;
use clap::{arg, command, ArgMatches, Args, Command, Parser};
use clap::{ArgAction, Subcommand};
use dialoguer::Confirm;
use env_logger::Env;
use log::{debug, error, info};
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
    // #[command(flatten)]
    // verbose: Verbosity<InfoLevel>,
}

#[derive(Subcommand)]
enum Commands {
    /// commit the current changes
    Commit(CommitArgs),
    /// force push to trigger a new workflow run
    Force(ForceArgs),
    /// set configuration values
    Config(ConfigArgs),
}

#[derive(Args)]
struct CommitArgs {
    /// add all unstaged changes before commiting
    #[arg(long, short, action = ArgAction::SetTrue)]
    all: bool,
    /// the workflow name to search for. NOTE this is NOT case sensitive
    #[arg(long, short)]
    workflow_name: Option<String>,
    // /// git commit message
    // #[arg(long, short)]
    // message: Vec<String>,
}

#[derive(Args)]
struct ForceArgs {
    /// the workflow name to search for. NOTE this is NOT case sensitive
    #[arg(long, short)]
    workflow_name: Option<String>,
}

#[derive(Args, Serialize, Deserialize, Default)]
struct ConfigArgs {
    #[arg(long)]
    hostname: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// github custom hostname
    hostname: Option<String>,
}

fn main() {
    let cli: ArgMatches = command!()
        .arg(
            arg!(-v --verbose "increase verbosity")
                .action(ArgAction::Count)
                .global(true),
        )
        .subcommand(
            Command::new("commit")
                .about("commit the current")
                .arg(arg!(-w --workflow <WORKFLOW_NAME> "name of the workflow to return"))
                .arg(arg!(-a --all "add all unstaged changes before commiting")),
        )
        .subcommand(
            Command::new("force")
                .about("force push to trigger a new workflow run")
                .arg(arg!(-w --workflow <WORKFLOW_NAME> "name of the workflow to return")),
        )
        .subcommand(
            Command::new("config")
                .about("set configuration values")
                .arg(
                    arg!(--hostname <HOSTNAME> "github hostname. ie mycorp.github.com")
                        .required(true),
                ),
        )
        .get_matches();

    let verbose_count = &cli.get_count("verbose");
    if verbose_count == &(1 as u8) {
        env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
    } else if verbose_count == &(2 as u8) {
        env_logger::Builder::from_env(Env::default().default_filter_or("warn,debug")).init();
    } else if verbose_count > &(2 as u8) {
        env_logger::Builder::from_env(Env::default().default_filter_or("warn,debug,trace")).init();
    } else {
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    }

    // CLI config values
    let config: Config = confy::load("gh-ac", None).unwrap();
    let gh = Gh::new(&config.hostname.as_deref());

    match cli.subcommand() {
        Some(("commit", args)) => {
            let arg_workflow_name = args.get_one::<String>("workflow");
            let arg_commit_all = args.get_one::<bool>("all");

            if *arg_commit_all.unwrap_or(&false) {
                match git::add_all() {
                    Some(e) => panic!("{}", e),
                    None => {}
                }
            } else if !git::check_staged_files() {
                info!("no staged files, exiting");
                return;
            }

            let selected_workflow_name = {
                if arg_workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    arg_workflow_name.clone().unwrap().to_string()
                }
            };
            debug!("user selected workflow name: {}", selected_workflow_name);

            let initial_workflow_run = gh.get_workflow_run_by_name(&selected_workflow_name);

            let commit_msg = git::commit(&None).unwrap();

            if commit_msg.is_none() {
                info!("no commit message was entered, exiting");
                return;
            }
            info!("commiting successful: {}", commit_msg.unwrap());

            git::push(false).unwrap();
            gh.check_for_new_workflow_run_by_id(&initial_workflow_run.unwrap());
        }
        Some(("force", args)) => {
            let arg_workflow_name = args.get_one::<String>("workflow");
            let selected_workflow_name = {
                if arg_workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    arg_workflow_name.clone().unwrap().to_string()
                }
            };

            if git::check_staged_files()
                && !Confirm::new()
                    .with_prompt("There are staged changes. Are you sure you want to force push?")
                    .default(false)
                    .interact()
                    .unwrap()
            {
                {
                    info!("Ok, aborting");
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
        Some(("config", args)) => {
            let arg_hostname = args.get_one::<String>("hostname");
            let config = ConfigArgs {
                hostname: arg_hostname.cloned(),
            };
            confy::store("gh-ac", None, config).unwrap();
            info!("config saved");
        }
        _ => {
            error!("no subcommand provided, exiting");
        }
    }
}
