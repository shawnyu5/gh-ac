mod gh;
mod git;
use std::{env, process};

use crate::gh::Gh;
use clap::{arg, command, Args, Command, Subcommand};
use clap::{ArgAction, Parser};
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
}

#[derive(Args)]
struct PushArgs {
    /// case insensitive name of the workflow to look for
    #[arg(short, long)]
    workflow: Option<String>,
    /// print out the workflow url instead of opening it in browser
    #[arg(long, action = clap::ArgAction::SetTrue)]
    url: Option<bool>,
}

#[derive(Args)]
struct ForceArgs {
    /// case insensitive name of the workflow to look for
    #[arg(short, long)]
    workflow: Option<String>,
    /// print out the workflow url instead of opening it in browser
    #[arg(long, action = clap::ArgAction::SetTrue)]
    url: Option<bool>,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct Config {
    /// github custom hostname
    hostname: Option<String>,
}

fn build_cli() -> Command {
    return command!()
        .arg(
            arg!(-v --verbose "increase verbosity")
                .action(ArgAction::Count)
                .global(true),
        )
        .subcommand(
            Command::new("push")
                .about("push all unpushed commits")
                .arg(arg!(-w --workflow <WORKFLOW_NAME> "name of the workflow to return"))
                .arg(arg!(--url "print out the workflow url instead of opening it in browser")),
        )
        .subcommand(
            Command::new("force")
                .about("force push to trigger a new workflow run")
                .arg(arg!(-w --workflow <WORKFLOW_NAME> "name of the workflow to return"))
                .arg(
                    arg!(--url "print out the workflow url instead of opening it in browser")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("config")
                .about("set configuration values")
                .arg(
                    arg!(--hostname <HOSTNAME> "github hostname. ie mycorp.github.com")
                        .required(true),
                ),
        );
}
fn main() {
    let cli = Cli::parse();
    return;

    let cli = build_cli().get_matches();

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
        Some(("push", args)) => {
            let arg_workflow_name = args.get_one::<String>("workflow");
            let arg_print_url = args.get_one::<bool>("url").unwrap_or_else(|| &false);

            let selected_workflow_name = {
                if arg_workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    arg_workflow_name.clone().unwrap().to_string()
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

            gh.check_for_new_workflow_run_by_id(&initial_workflow_run, &arg_print_url)
        }
        Some(("force", args)) => {
            let arg_workflow_name = args.get_one::<String>("workflow");
            let arg_print_url = args.get_one::<bool>("url").unwrap_or_else(|| &false);

            let selected_workflow_name = {
                if arg_workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    arg_workflow_name.clone().unwrap().trim().to_string()
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

            gh.check_for_new_workflow_run_by_id(&initial_workflow_run, arg_print_url)
        }
        Some(("config", args)) => {
            let arg_hostname = args.get_one::<String>("hostname");
            let config = Config {
                hostname: arg_hostname.cloned(),
            };
            confy::store("gh-ac", None, config).unwrap();
            info!("config saved");
        }
        _ => {
            match Command::print_help(&mut build_cli()) {
                Ok(_) => {}
                Err(_) => {
                    error!("Failed to print help");
                }
            };
        }
    }
}
