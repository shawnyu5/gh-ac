use anyhow::{anyhow, Result};
use dialoguer::{console::Term, theme::ColorfulTheme, FuzzySelect};
use log::{debug, error, info, trace};
use serde_derive::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::{
    env,
    fmt::Display,
    process::{self, Command},
};

#[derive(Debug, Clone)]
pub struct Gh {
    /// custom github api hostname
    hostname: Option<String>,
    /// if the custom hostname should be used in this repo
    should_use_custom_hostname: bool,
}

impl Default for Gh {
    fn default() -> Self {
        Self {
            hostname: Default::default(),
            should_use_custom_hostname: false,
        }
    }
}
impl Display for Workflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.path)
    }
}
impl Gh {
    pub fn new(hostname: Option<String>) -> Gh {
        let mut gh = Gh {
            hostname,
            ..Default::default()
        };
        gh.check_should_use_custom_hostname();
        return gh;
    }

    /// determine if the custom hostname should be used in this repo. Modifies `self.should_use_custom_hostname`
    ///
    /// It does this by running `gh api users`, if the command fails, then it is assumed that the custom hostname should be used
    fn check_should_use_custom_hostname(&mut self) {
        // let args = self.construct_gh_api_args(&mut vec!["/repos/{owner}/{repo}/actions/runs"]);
        let args = vec!["api", "/repos/{owner}/{repo}/actions/runs"];
        let cmd = Command::new("gh").args(args).output().unwrap();
        trace!("gh api /repos/{{owner}}/{{repo}}/actions/runs: {:?}", cmd);

        if cmd.status.success() {
            self.should_use_custom_hostname = false
        } else {
            self.should_use_custom_hostname = true
        }
        debug!(
            "Using custom hostname: {}",
            &self.should_use_custom_hostname
        );
    }
    /// construct arguments for `gh api`, including the optional `--hostname` if applicable
    /// * `args`: the args to pass to `gh api <args>`
    ///
    /// returns the args to pass to `gh api <args>`
    fn construct_gh_api_args<'a>(&'a self, args: &mut Vec<&'a str>) -> Vec<&'a str> {
        if self.should_use_custom_hostname {
            match &self.hostname {
                Some(hostname) => {
                    debug!("appending custom hostname to gh command");
                    let mut gh_args = vec!["api", "--hostname", hostname.as_str()];
                    gh_args.append(args);
                    return gh_args;
                }
                None => {
                    panic!(
                        "no hostname specified. Add a hostname using `config --hostname <HOSTNAME>"
                    );
                }
            }
        } else {
            let mut gh_args = vec!["api"];
            gh_args.append(args);
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
        trace!(
            "gh api /repos/{{owner}}/{{repo}}/actions/runs: {:?}",
            output
        );

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let workflow_runs = serde_json::from_str::<WorkflowRuns>(&stdout)?;

            for r in workflow_runs.workflow_runs.unwrap() {
                if &r.name.to_lowercase() == &name.to_lowercase() {
                    return Ok(r);
                }
            }
            return Err(anyhow!("No workflow with name {} found...", name));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // if there was no hostname passed in, retry with a hostname
            println!("Command failed with error: {}", stderr);
            return Err(anyhow!("failed getting actions run..."));
        }
    }

    /// get all workflows of a repo
    ///
    /// returns all active workflows of a repo
    pub fn repo_workflows(&self) -> Result<Workflows> {
        let args = self.construct_gh_api_args(&mut vec!["/repos/{owner}/{repo}/actions/workflows"]);

        trace!(
            "gh api /repos/{{owner}}/{{repo}}/actions/workflows: {:?}",
            args
        );

        let output = Command::new("gh")
            .args(&args)
            .output()
            .expect("to get workflow runs from `gh`");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut workflows = serde_json::from_str::<Workflows>(&stdout)?;
            workflows.workflows = workflows
                .workflows
                .into_iter()
                .filter(|e| e.state.clone().unwrap_or("".to_string()) == "active")
                .collect();

            // update total count after filtering
            workflows.total_count = workflows.workflows.len();
            return Ok(workflows);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("failed getting repo workflows...: {}", stderr));
        }
    }

    /// list workflow runs for a workflow
    ///
    /// * `workflow_id`: the id of the workflow to get workflow runs for
    pub fn list_workflow_runs_for_workflow(&self, workflow_id: &i64) -> Result<SingleWorkflowRuns> {
        let url =
            format!("/repos/{{owner}}/{{repo}}/actions/workflows/{workflow_id}/runs?per_page=500");
        let args = self.construct_gh_api_args(&mut vec![url.as_str()]);
        trace!("gh api {}: {:?}", url, args);

        let output = Command::new("gh").args(&args).output()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(serde_json::from_str::<SingleWorkflowRuns>(&stdout).unwrap());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("failed getting workflow runs...: {}", stderr));
        }
    }

    /// check for a new workflow run with an id new workflow runs
    ///
    /// * `old_workflow_run`: the last workflow run that was observed in the repo
    pub fn check_for_new_workflow_run_by_id(
        &self,
        old_workflow_run: &WorkflowRun,
        print_url: &bool,
    ) {
        let mut spinner =
            Spinner::with_timer(Spinners::Star, "Wating for workflow to start...".into());
        info!("sleep for 3 seconds");
        std::thread::sleep(std::time::Duration::from_secs(3));
        loop {
            let current_workflow_run = self
                .get_workflow_run_by_name(&old_workflow_run.name)
                .unwrap_or_default();

            if old_workflow_run == &current_workflow_run {
                info!("no workflow run has started...");
                info!("waiting for 3 seconds");
                std::thread::sleep(std::time::Duration::from_secs(3));
                continue;
            }
            if *print_url {
                spinner.stop_and_persist("🗸", format!("{}", &current_workflow_run.html_url));
                info!("workflow found: {}", &current_workflow_run.html_url);
            } else {
                match Command::new(get_browser())
                    .arg(&current_workflow_run.html_url)
                    .output()
                {
                    Ok(_) => {
                        spinner.stop_and_persist(
                            "🗸",
                            format!("Opening {} in browser", &current_workflow_run.html_url),
                        );
                        info!("Opening {} in browser", &current_workflow_run.html_url);
                    }
                    Err(_) => {
                        error!(
                        "failed to open browser. Please open the following url in your browser: {}",
                        &current_workflow_run.html_url
                    )
                    }
                };
            }

            break;
        }
    }

    /// If there are more than 1 workflows defined in the repo, prompt the user for which workflow they would like to use
    /// prompt: The prompt to display to the user. Defaults to `Select a workflow`
    ///
    /// returns the user selected workflow object
    pub fn select_workflow(&self, prompt: Option<&str>) -> Workflow {
        let workflows = self.repo_workflows().unwrap_or_default();
        trace!("workflows: {:?}", workflows);

        if &workflows.total_count > &1 {
            let selection_index = match FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt(prompt.unwrap_or_else(|| "Select a workflow"))
                .items(&workflows.workflows)
                .default(0)
                .interact_on_opt(&Term::stdout())
                .unwrap()
            {
                Some(idx) => idx,
                None => {
                    info!("no workflow selected, exiting");
                    process::exit(1);
                }
            };

            workflows.workflows[selection_index].clone()
        } else if &workflows.workflows.len() == &1 {
            workflows.workflows[0].clone()
        } else {
            error!("no workflows found in repo, exiting");
            process::exit(0);
        }
    }

    pub fn dispatch_workflow_run(
        &self,
        reference: String,
        workflow_name: &String,
        body: &Option<Vec<String>>,
    ) -> Result<()> {
        let mut spinner = Spinner::with_timer(Spinners::Dots9, "Dispatching workflow".into());
        let mut args: Vec<String> = vec![
            "workflow".to_string(),
            "run".to_string(),
            workflow_name.to_string(),
            "--ref".to_string(),
            reference,
        ];

        if let Some(body) = body {
            let body_args: Vec<String> = body
                .into_iter()
                .flat_map(|v| vec!["-f".to_string(), v.to_owned()])
                .collect();
            args.extend(body_args);
        }
        debug!("Gh args: {:?}", args);

        let output = Command::new("gh")
            .args(args)
            .output()
            .expect("to run workflow");

        if output.status.success() {
            spinner.stop_and_persist("🗸", "Workflow run dispatched successfully".into());
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            spinner.stop_with_newline();
            return Err(anyhow!("Error running workflow: {}", stderr));
        }
    }
    /// delete a specific run of a workflow
    ///
    /// * `run_id`: the workflow run id to delete
    pub fn delete_workflow_run(&self, run_id: i64) -> Result<()> {
        let url = format!("/repos/{{owner}}/{{repo}}/actions/runs/{}", run_id);
        let args = self.construct_gh_api_args(&mut vec![url.as_str(), "--method", "DELETE"]);
        debug!("Deleting workfow run: {}", run_id);
        debug!("Gh args: {:?}", args);

        let output = Command::new("gh").args(args).output()?;

        if output.status.success() {
            return Ok(());
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("failed deleting workflow run logs: {}", stderr));
        }
    }
}

/// get the default browser
fn get_browser() -> String {
    return env::var("BROWSER").unwrap_or_else(|_| "open".to_string());
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
    pub display_title: Option<String>,
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
    pub total_count: usize,
    pub workflows: Vec<Workflow>,
}

/// all workflow runs for a single workflow
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SingleWorkflowRuns {
    pub total_count: usize,
    pub workflow_runs: Vec<Workflow>,
}

/// a single workflow of a repo
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Workflow {
    pub id: i64,
    pub node_id: String,
    pub name: String,
    pub path: String,
    pub state: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub url: String,
    pub html_url: String,
}
