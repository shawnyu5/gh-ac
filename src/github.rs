use anyhow::{anyhow, Result};
use octocrab::{models::workflows::Run, Octocrab, Page};
use std::process::Command;

use crate::gh::Gh;

#[derive(Debug, Clone)]
pub struct Github {
    /// an authenticated instance of Octocrab
    octocrab: Octocrab,
}

impl Default for Github {
    fn default() -> Self {
        Self {
            octocrab: Default::default(),
        }
    }
}

impl Github {
    /// Create a new instance of Github, with a
    ///
    /// * `hostname`: custom github api endpoint
    pub fn new(hostname: Option<String>) -> Self {
        let pat = Gh::get_pat_token().unwrap();
        let mut octocrab_builder = Octocrab::builder().personal_token(pat);
        if let Some(hostname) = hostname {
            octocrab_builder = octocrab_builder.base_uri(hostname).expect("valid base url");
        }
        Self {
            octocrab: octocrab_builder.build().unwrap(),
        }
    }
    pub async fn get_workflow_run_by_name(&self, run: &Run) -> Result<Page<Run>> {
        Ok(self
            .octocrab
            .workflows(
                run.repository.owner.clone().unwrap().login,
                &run.repository.name,
            )
            .list_runs(run.id.to_string())
            .send()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use crate::gh::Gh;

    #[tokio::test]
    async fn test_get_workflow_run_by_name() {
        let gh = Gh::new(None);
    }
}
