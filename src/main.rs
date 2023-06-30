mod gh;
mod git;
use std::process;

use crate::gh::Gh;
use clap::Parser;
use clap::{arg, command, Args, Subcommand};
use dialoguer::Confirm;
use env_logger::Env;
use log::{error, info};
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbosity: u8,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// push all unpushed commits
    Push(PushArgs),
    /// force push to trigger new workflow run(s)
    Force(ForceArgs),
    /// set configuration values
    Config(ConfigArgs),
}

#[derive(Args)]
struct PushArgs {
    /// case insensitive name of the workflow to look for
    #[arg(short, long = "workflow")]
    workflow_name: Option<String>,
    /// print out the workflow url instead of opening it in browser
    #[arg(long, action = clap::ArgAction::SetTrue)]
    url: Option<bool>,
}

#[derive(Args)]
struct ForceArgs {
    /// case insensitive name of the workflow to look for
    #[arg(short, long = "workflow")]
    workflow_name: Option<String>,
    /// print out the workflow url instead of opening it in browser
    #[arg(long, action = clap::ArgAction::SetTrue)]
    url: Option<bool>,
}

#[derive(Args)]
struct ConfigArgs {
    /// github custom hostname
    #[arg(long)]
    hostname: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// github custom hostname
    hostname: Option<String>,
}

fn main() {
    // CLI config values
    let config: Config = confy::load("gh-ac", None).unwrap();
    let gh = Gh::new(&config.hostname.as_deref());
    let cli = Cli::parse();

    match cli.verbosity {
        1 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("warn")).init();
        }
        2 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("warn,debug")).init();
        }
        3 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("warn,debug,trace"))
                .init();
        }
        _ => {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        }
    }

    match cli.command {
        Commands::Push(args) => {
            let selected_workflow_name = {
                if args.workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    args.workflow_name.clone().unwrap().to_string()
                }
            };

            let initial_workflow_run = gh
                .get_workflow_run_by_name(&selected_workflow_name)
                .unwrap();

            match git::push(false) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to push changes: {}", e.to_string());
                    process::exit(1);
                }
            }

            gh.check_for_new_workflow_run_by_id(
                &initial_workflow_run,
                &args.url.unwrap_or_else(|| false),
            )
        }
        Commands::Force(args) => {
            let selected_workflow_name = {
                if args.workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    args.workflow_name.unwrap().trim().to_string()
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
            git::push(true).expect("failed to push");

            gh.check_for_new_workflow_run_by_id(
                &initial_workflow_run,
                &args.url.unwrap_or_else(|| false),
            )
        }
        Commands::Config(args) => {
            let config = Config {
                hostname: Some(args.hostname),
            };
            confy::store("gh-ac", None, config).unwrap();
            info!("config saved");
        }
    }
}
