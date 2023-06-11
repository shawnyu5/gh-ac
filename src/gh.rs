use crate::Config;
use anyhow::{anyhow, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, FuzzySelect};
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::Display,
    process::{self, Command},
};

#[derive(Default, Debug)]
pub struct Gh<'a> {
    /// custom github api hostname
    hostname: Option<&'a str>,
    /// if the custom hostname should be used in this repo
    should_use_custom_hostname: bool,
}
impl Display for Workflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.path)
    }
}
impl Gh<'_> {
    pub fn new<'a>(hostname: &Option<&'a str>) -> Gh<'a> {
        let mut gh = Gh {
            hostname: *hostname,
            ..Default::default()
        };
        gh.check_should_use_custom_hostname();
        return gh;
    }

    /// determine if the custom hostname should be used in this repo. Modifies `self.should_use_custom_hostname`
    ///
    /// It does this by running `gh api users`, if the command fails, then it is assumed that the custom hostname should be used
    fn check_should_use_custom_hostname(&mut self) {
        let args = self.construct_gh_api_args(&mut vec!["users"]);
        match Command::new("gh").args(args).output() {
            Ok(_) => self.should_use_custom_hostname = false,
            Err(_) => self.should_use_custom_hostname = true,
        }
    }
    /// construct arguments for `gh api`, including the optional `--hostname` if applicable
    ///
    /// * `args`: the args to pass to `gh api <args>`
    ///
    /// returns the args to pass to `gh api <args>`
    fn construct_gh_api_args<'a>(&'a self, args: &mut Vec<&'a str>) -> Vec<&'a str> {
        return if self.should_use_custom_hostname {
            match self.hostname {
                Some(hostname) => vec!["api", "--hostname", hostname],
                None => {
                    let mut gh_args = vec!["api"];
                    gh_args.append(args);
                    return gh_args;
                }
            }
        } else {
            let mut gh_args = vec!["api"];
            gh_args.append(args.as_mut());
            return gh_args;
        };
    }
    /// get the latest workflow run for a workflow by id
    /// `id`: workflow id to search for
    ///
    /// returns the latest workflow run for a workflow with `id`
    pub fn get_workflow_run_by_name(&self, name: &String) -> Result<WorkflowRun> {
        let args = self.construct_gh_api_args(&mut vec!["/repos/{owner}/{repo}/actions/runs"]);

        let output = Command::new("gh")
            .args(&args)
            .output()
            .expect("to get workflow runs from `gh`");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let workflow_runs = serde_json::from_str::<WorkflowRuns>(&stdout)?;

            for r in workflow_runs.workflow_runs.unwrap() {
                if &r.name == name {
                    return Ok(r);
                }
            }
            return Err(anyhow!("No workflow with name {} found...", name));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // if there was no hostname passed in, retry with a hostname
            eprintln!("Command failed with error: {}", stderr);
            return Err(anyhow!("failed getting actions run..."));
        }
    }

    /// get all workflows of a repo
    ///
    /// `use_custom_hostname`: whether or not to use the custom hostname from the config file
    /// returns all workflows of a repo
    pub fn repo_workflows(&self) -> Result<Workflows> {
        let args = self.construct_gh_api_args(&mut vec!["/repos/{owner}/{repo}/actions/workflows"]);

        let output = Command::new("gh")
            .args(&args)
            .output()
            .expect("to get workflow runs from `gh`");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(serde_json::from_str::<Workflows>(&stdout)?);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("failed getting repo workflows...: {}", stderr));
        }
    }

    /// check for a new workflow run with an id new workflow runs
    ///
    /// * `old_workflow_run`: the last workflow run we are looking in the repo
    pub fn check_for_new_workflow_run_by_id(&self, old_workflow_run: &WorkflowRun) {
        println!("waiting for 5 seconds");
        std::thread::sleep(std::time::Duration::from_secs(5));
        loop {
            let current_workflow_run = self
                .get_workflow_run_by_name(&old_workflow_run.name)
                .unwrap_or_default();

            if old_workflow_run == &current_workflow_run {
                println!("no workflow run has started...");
                println!("waiting for 5 seconds");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
            println!("{}", current_workflow_run.html_url);
            break;
        }
    }

    /// If there are more than 1 workflows defined in the repo, prompt the user for which workflow they would like to use
    ///
    /// returns the user selected workflow id
    pub fn select_workflow_name(&self) -> String {
        let workflows = self.repo_workflows().unwrap_or_default();

        // if there are more than one workflow in the repo, ask the user which one they would like
        if &workflows.total_count > &1 {
            let selection_index = FuzzySelect::with_theme(&ColorfulTheme::default())
                .items(&workflows.workflows)
                .default(0)
                .interact_on_opt(&Term::stdout())
                .unwrap()
                .unwrap();

            workflows.workflows[selection_index].name.clone()
        } else if &workflows.total_count == &1 {
            workflows.workflows[0].name.clone()
        } else {
            println!("no workflows found in repo, exiting");
            process::exit(0);
        }
    }
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

/// all workflows of a repo
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflows {
    pub total_count: i64,
    pub workflows: Vec<Workflow>,
}

/// a single workflow of a repo
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub path: String,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub html_url: String,
}
