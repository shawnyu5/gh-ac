package utils

import (
	"encoding/json"
	"errors"
	"github.com/briandowns/spinner"
	"github.com/charmbracelet/log"
	"github.com/cli/go-gh/v2"
	"github.com/google/go-github/v61/github"
	"math/rand/v2"
	"os"
	"os/exec"
	"strings"
	"time"
)

// GetWorkflowRunByName get the latest workflow run, searching by name. `name` is case-insensitive
//
// Will return an error if no workflow by the name `name` is found
func GetWorkflowRunByName(name string) (*github.WorkflowRun, error) {
	stdout, _, err := gh.Exec("api", "/repos/{owner}/{repo}/actions/runs")
	if err != nil {
		return nil, errors.New("Failed to get workflow run: " + err.Error())
	}

	var workflowRuns github.WorkflowRuns
	err = json.Unmarshal(stdout.Bytes(), &workflowRuns)
	if err != nil {
		return nil, errors.New("Failed to parse workflow runs: " + err.Error())
	}

	for _, workflowRun := range workflowRuns.WorkflowRuns {
		if strings.ToLower(workflowRun.GetName()) == strings.ToLower(name) {
			return workflowRun, nil
		}
	}

	return nil, errors.New("no workflow found")
}

// TrackNewWorkflowRun will trigger a workflow run, by calling `trigger()`, and look for a new workflow run, with the
// name `workflowName`
//
// `trigger` MUST trigger a new workflow run
func TrackNewWorkflowRun(workflowName string, trigger func()) (*github.WorkflowRun, error) {
	stdout, _, err := gh.Exec("api", "/repos/{owner}/{repo}/actions/workflows")
	if err != nil {
		log.Fatalf("Failed to get repo workflows: %w", err)
	}

	// all workflows defined in current repo
	var repoWorkflows github.Workflows
	err = json.Unmarshal(stdout.Bytes(), &repoWorkflows)
	if err != nil {
		return nil, err
	}

	//var workflowName string
	//
	//if flags.workflowName != "" {
	//	workflowName = flags.workflowName
	//} else {
	//	// If there are multiple workflows defined for the repo, prompt user for which workflow they would like to use
	//	if repoWorkflows.GetTotalCount() > 1 {
	//		var workflowNames []string
	//		for _, workflow := range repoWorkflows.Workflows {
	//			workflowNames = append(workflowNames, workflow.GetName())
	//		}
	//
	//		idx, err := fuzzyfinder.Find(workflowNames, func(i int) string {
	//			return workflowNames[i]
	//		})
	//
	//		if errors.Is(err, fuzzyfinder.ErrAbort) {
	//			log.Debug("User aborted during selection")
	//			os.Exit(0)
	//		} else if err != nil {
	//			log.Fatalf("Failed to get selection: %w", err)
	//		}
	//
	//		log.Debugf("Selected workflowName: %s", workflowNames[idx])
	//
	//		workflowName = workflowNames[idx]
	//	} else {
	//		// Otherwise if there are only a single workflow, use that one
	//		workflowName = repoWorkflows.Workflows[0].GetName()
	//	}
	//}

	stdout, _, err = gh.Exec("api", "/repos/{owner}/{repo}/actions/runs")
	if err != nil {
		log.Fatalf("Failed to get workflowName run: %s", err)
	}

	initialWorkflowRun, err := GetWorkflowRunByName(workflowName)
	if err != nil {
		log.Fatalf("Failed to get initial workflowName run: %s", err)
	}

	// trigger new workflow run
	trigger()

	//log.Infof("Waiting for new workflow run '%s' to start...", workflowName)
	for {
		log.Debug("Sleep for 3s")
		time.After(3 * time.Second)

		stdout, _, err = gh.Exec("api", "/repos/{owner}/{repo}/actions/runs")
		if err != nil {
			log.Fatalf("Failed to get workflowName run: %s", err)
		}

		newWorkflowRun, err := GetWorkflowRunByName(workflowName)
		if err != nil {
			log.Fatalf("Failed to retrieve workflowName run after new workflowName trigger: %s", err)
		}

		// If ID of previous vs new workflowName run doesn't match, that means a new workflow has started
		if newWorkflowRun.GetID() != initialWorkflowRun.GetID() {
			log.Debug("Found new workflowName")
			return newWorkflowRun, nil
		}
	}
}

// OpenInBrowser passes a list of arguments to the system browser configured by `$BROWSER` env var
func OpenInBrowser(args []string) error {
	log.Infof("Opening %s in browser", args)
	err := exec.Command(os.Getenv("BROWSER"), args...).Run()
	return err
}

// RandomSpinner creates a random spinner
func RandomSpinner(suffix string) *spinner.Spinner {
	s := spinner.New(spinner.CharSets[rand.IntN(90)], 100*time.Millisecond)
	s.Suffix = suffix
	return s
}
