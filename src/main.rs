mod gh;
mod git;
use std::sync::mpsc;
use std::{process, thread};

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
    /// create a workflow dispatch event
    Dispatch(DispatchArgs),
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
struct DispatchArgs {
    /// case insensitive name of the workflow to look for
    #[arg(short, long = "workflow")]
    workflow_name: Option<String>,
    /// branch or commit reference
    #[arg(long = "ref")]
    reference: Option<String>,
    /// input to pass to the workflow, in the form `KEY=VALUE`
    #[arg(short = 'f', long = "form")]
    body: Option<Vec<String>>,
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
    let gh = Gh::new(config.hostname);
    let cli = Cli::parse();

    match cli.verbosity {
        1 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        }
        2 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("info,warn")).init();
        }
        3 => {
            env_logger::Builder::from_env(Env::default().default_filter_or("info,warn,debug"))
                .init();
        }
        4 => {
            env_logger::Builder::from_env(
                Env::default().default_filter_or("info,warn,debug,trace"),
            )
            .init();
        }
        _ => {
            env_logger::Builder::from_env(Env::default().default_filter_or("")).init();
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

            let action = |_: &String| match git::push(false) {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to push changes: {}", e.to_string());
                    process::exit(1);
                }
            };
            track_new_workflow(
                action,
                &gh,
                selected_workflow_name,
                args.url.unwrap_or(false),
            );
        }
        Commands::Force(args) => {
            if git::check_staged_files()
                && !Confirm::new()
                    .with_prompt("There are staged changes. Are you sure you want to force push?")
                    .default(false)
                    .interact()
                    .unwrap()
            {
                info!("Ok, aborting");
                return;
            }

            let selected_workflow_name = {
                if args.workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    args.workflow_name.unwrap().trim().to_string()
                }
            };

            let action = |_: &String| {
                git::commit_amend_no_edit().unwrap();
                git::push(true).expect("failed to push");
            };

            track_new_workflow(
                action,
                &gh,
                selected_workflow_name,
                args.url.unwrap_or(false),
            );
        }
        Commands::Dispatch(args) => {
            let selected_workflow_name = {
                if args.workflow_name.is_none() {
                    gh.select_workflow_name()
                } else {
                    args.workflow_name.clone().unwrap().to_string()
                }
            };

            let handler = |selected_workflow_name: &String| {
                match gh.dispatch_workflow_run(
                    args.reference.unwrap_or_else(|| git::current_branch_name()),
                    &selected_workflow_name,
                    &args.body,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Failed to dispatch workflow: {}", e.to_string());
                        process::exit(1);
                    }
                };
            };
            track_new_workflow(
                handler,
                &gh,
                selected_workflow_name,
                args.url.unwrap_or(false),
            );
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
/// Sends a request to GH api for the current list of workflows, then calls `func`, and watches for new workflows that gets triggered
///
/// * `trigger_workflow_run`: a function that accepts a workflow name as the single argument. It should perform an action that triggers new workflow runs
/// * `gh`: the Gh instance
/// * `workflow_name`: the workflow to look for
/// * `print_url`: if the url to the workflow should be printed out instead of opened in the browser
fn track_new_workflow<F>(trigger_workflow_run: F, gh: &Gh, workflow_name: String, print_url: bool)
where
    F: FnOnce(&String),
{
    let (sender, receiver) = mpsc::channel();

    {
        let gh = gh.clone();
        let selected_workflow_name = workflow_name.clone();
        thread::spawn(move || {
            let initial_workflow_run = gh
                .get_workflow_run_by_name(&selected_workflow_name)
                .unwrap();
            sender.send(initial_workflow_run).unwrap();
        });
    }

    trigger_workflow_run(&workflow_name);

    gh.check_for_new_workflow_run_by_id(&receiver.recv().unwrap(), &print_url);
}
