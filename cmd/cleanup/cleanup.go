package cleanup

import (
	"errors"
	"fmt"

	"github.com/charmbracelet/log"
	"github.com/google/go-github/v61/github"
	"github.com/shawnyu5/gh-ac/cmd"
	"github.com/shawnyu5/gh-ac/gh"
	"github.com/shawnyu5/gh-ac/utils"
	"github.com/spf13/cobra"
)

type cmdFlags struct {
	// Name of workflowName
	workflowName string
}

var flags cmdFlags

// cleanupCmd represents the cleanup command
var cleanupCmd = &cobra.Command{
	Use:   "cleanup",
	Short: "Clean up workflow run history for a specific workflow",
	Run: func(cmd *cobra.Command, args []string) {
		var workflowName string

		if flags.workflowName != "" {
			workflowName = flags.workflowName
		} else {
			name, err := utils.SelectRepoWorkflowName()
			if err != nil {
				log.Fatalf("Failed to select target workflow: %s", err)
			}
			workflowName = *name
		}

		workflow, err := findWorkflowByName(workflowName)
		if err != nil {
			log.Fatalf("Failed to find workflow with name %s: %s", workflowName, err)
		}

		log.Debugf("Workflow ID: %d", workflow.GetID())
		workflowRuns, err := listRunsForWorkflow(workflow.GetID())
		if err != nil {
			log.Fatalf("Failed to get workflow runs for workflow %s: %s", workflow.GetName(), err)
		}

		//for _, workflow := range workflowRuns {
		//	log.Debugf("workflow runs for workflow %s: %+v", workflow.GetName(), workflow.GetID())
		//}
		//
		//return

		s := utils.RandomSpinner("Deleting workflow runs...")
		s.Start()

		//var wg sync.WaitGroup

		for _, workflowRun := range workflowRuns {
			log.Debugf("Deleting workflow with ID: %d", workflow.GetID())
			err := deleteWorkflowRun(workflowRun.GetID())
			if err != nil {
				log.Errorf("Failed to delete workflow: %s", err)
			}
		}
		s.Stop()
	},
}

func init() {
	cmd.RootCmd.AddCommand(cleanupCmd)
	cleanupCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name to delete the history of")
}

// listRunsForWorkflow fetches workflow runs for a particular workflow
func listRunsForWorkflow(workflowID int64) ([]*github.WorkflowRun, error) {
	//"/repos/{{owner}}/{{repo}}/actions/workflows/{workflow_id}/runs?{}",
	page := 1
	var workflowRuns []*github.WorkflowRun
	for {
		workflowRun, err := gh.New[github.WorkflowRuns]().
			Arg("api").
			Arg(fmt.Sprintf("/repos/{owner}/{repo}/actions/workflows/%d/runs?per_page=100&page=%d", workflowID, page)).
			Exec()
		if err != nil {
			return nil, err
		}

		if len(workflowRun.WorkflowRuns) == 0 {
			break
		}
		workflowRuns = append(workflowRuns, workflowRun.WorkflowRuns...)
		page++
	}

	return workflowRuns, nil

}

// findWorkflowByName finds a workflow by its name. Returns a workflow object
func findWorkflowByName(name string) (*github.Workflow, error) {
	// All defined workflows in the repo
	var repoWorkflowDefinitions []*github.Workflow
	page := 1
	for {
		workflows, err := gh.New[github.Workflows]().
			Arg("api").
			Arg(fmt.Sprintf("/repos/{owner}/{repo}/actions/workflows?per_page=100&page=%d", page)).
			AppendHostname(true).
			Exec()
		if err != nil {
			return nil, err
		}
		if len(workflows.Workflows) == 0 {
			break
		}
		repoWorkflowDefinitions = append(repoWorkflowDefinitions, workflows.Workflows...)
		page++
	}

	for _, workflow := range repoWorkflowDefinitions {
		if workflow.GetName() == name {
			return workflow, nil
		}

	}
	return nil, errors.New("no workflow found")
}

// deleteWorkflowRun deletes a workflow run by the run ID
func deleteWorkflowRun(workflowID int64) error {
	_, err := gh.New[any]().
		Arg("api").
		Arg(fmt.Sprintf("/repos/{owner}/{repo}/actions/runs/%d", workflowID)).
		Arg("--method").
		Arg("DELETE").
		ParseOutputJson(false).
		Exec()

	return err
}
