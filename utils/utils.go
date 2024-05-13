package utils

import (
	"errors"
	"fmt"
	"github.com/briandowns/spinner"
	"github.com/charmbracelet/log"
	"github.com/google/go-github/v61/github"
	"github.com/ktr0731/go-fuzzyfinder"
	"github.com/shawnyu5/gh-ac/gh"
	// TODO: use V2 once this issue is solved. v2 is not included in go 1.21
	// https://github.com/cli/gh-extension-precompile/issues/50
	"math/rand"
	"os"
	"os/exec"
	"strings"
	"time"
)

// GetWorkflowRunByName get the latest workflow run, searching by name. `name` is case-insensitive
//
// Will return an error if no workflow with `name` is found
func GetWorkflowRunByName(name string) (*github.WorkflowRun, error) {
	workflowRuns, err := gh.New[github.WorkflowRuns]().Arg("api").Arg("/repos/{owner}/{repo}/actions/runs").AppendHostName().Exec()
	if err != nil {
		return nil, err
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
// `trigger` MUST trigger a new workflow run.
func TrackNewWorkflowRun(workflowName string, trigger func()) (*github.WorkflowRun, error) {
	initialWorkflowRun, err := GetWorkflowRunByName(workflowName)
	if err != nil {
		log.Fatalf("Failed to get initial workflowName run: %s", err)
	}

	// trigger new workflow run
	trigger()

	for {
		log.Debug("Sleep for 3s")
		time.After(3 * time.Second)

		newWorkflowRun, err := GetWorkflowRunByName(workflowName)
		if err != nil {
			log.Fatalf("Failed to retrieve workflow run after new workflow trigger: %s", err)
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
	s := spinner.New(spinner.CharSets[rand.Intn(90)], 100*time.Millisecond)
	s.Suffix = suffix
	return s
}

// SelectRepoWorkflowName gets all defined workflows in the repo.
//
// If there are more than 1 workflow defined, prompt the user to select a workflow. Otherwise return the only workflow
// name in the repo
func SelectRepoWorkflowName() (workflowName *string, err error) {
	// All defined workflows in the repo
	var repoWorkflowDefinitions []*github.Workflow
	page := 1
	for {
		workflows, err := gh.New[github.Workflows]().Arg("api").Arg(fmt.Sprintf("/repos/{owner}/{repo}/actions/workflows?per_page=100&page=%d", page)).AppendHostName().Exec()
		if err != nil {
			return nil, err
		}
		if len(workflows.Workflows) == 0 {
			break
		}
		repoWorkflowDefinitions = append(repoWorkflowDefinitions, workflows.Workflows...)
		page++
	}

	// If there are multiple workflows defined for the repo, prompt user for which workflow they would like to use
	if len(repoWorkflowDefinitions) > 1 {
		var workflowNames []string
		for _, workflowDefinition := range repoWorkflowDefinitions {
			workflowNames = append(workflowNames, workflowDefinition.GetName())
		}

		idx, err := fuzzyfinder.Find(workflowNames, func(i int) string {
			return workflowNames[i]
		})

		if errors.Is(err, fuzzyfinder.ErrAbort) {
			log.Debug("User aborted during selection")
			os.Exit(0)
		} else if err != nil {
			log.Fatalf("Failed to get selection: %w", err)
		}

		log.Debugf("Selected workflow name: %s", workflowNames[idx])

		workflowName = &workflowNames[idx]
	} else {
		// Otherwise if there are only a single workflow, use that one
		name := repoWorkflowDefinitions[0].GetName()
		workflowName = &name
	}

	return workflowName, nil
}
