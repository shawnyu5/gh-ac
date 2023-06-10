use crate::Config;
use anyhow::{anyhow, Result};
use serde_derive::{Deserialize, Serialize};
use std::process::Command;

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
/// get all workflow runs of a repo
/// * `hostname` - custom gh hostname
pub fn get_workflow_runs<'a>(hostname: Option<&'a str>) -> Result<WorkflowRuns> {
    let conf = confy::load::<Config>("gh-ac", None).expect("config file");
    let args = {
        match hostname {
            Some(hostname) => {
                vec![
                    "api",
                    "/repos/{owner}/{repo}/actions/runs",
                    "--hostname",
                    hostname,
                ]
            }
            None => vec!["api", "/repos/{owner}/{repo}/actions/runs"],
        }
    };

    let output = Command::new("gh")
        .args(args.clone())
        .output()
        .expect("to get workflow runs from `gh`");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(serde_json::from_str::<WorkflowRuns>(&stdout)?);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // if there was no hostname passed in, retry with a hostname
        if hostname.is_none() {
            eprintln!(
                "Command failed with error:\n{}\nRetrying with custom gh hostname",
                stderr
            );
            return get_workflow_runs(conf.gh_hostname.as_deref());
        }
        return Err(anyhow!("failed getting actions run...: {}", stderr));
    }
}
