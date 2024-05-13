package push

import (
	"encoding/json"
	"errors"
	"github.com/charmbracelet/log"
	"github.com/cli/go-gh/v2"
	"github.com/google/go-github/v61/github"
	"github.com/ktr0731/go-fuzzyfinder"
	"github.com/shawnyu5/gh-ac/cmd"
	"github.com/shawnyu5/gh-ac/git"
	"github.com/shawnyu5/gh-ac/utils"
	"github.com/spf13/cobra"
	"os"
)

type cmdFlags struct {
	// Name of workflowName
	workflowName string
	// Toggle print the URL to workflowName instead of opening it in browser. Defaults to false
	printUrl bool
}

var flags cmdFlags

// pushCmd represents the push command
var pushCmd = &cobra.Command{
	Use:   "push",
	Short: "Push current changes and open workflowName in browser",
	Long: `Push current changes and open workflowName in browser

If no workflowName run as been started, this command will wait indefinite until a new workflowName run is started`,
	RunE: func(cmd *cobra.Command, args []string) error {
		stdout, _, err := gh.Exec("api", "/repos/{owner}/{repo}/actions/workflows")
		if err != nil {
			log.Fatalf("Failed to get repo workflows: %w", err)
		}

		// all workflows defined in current repo
		var repoWorkflows github.Workflows
		err = json.Unmarshal(stdout.Bytes(), &repoWorkflows)
		if err != nil {
			return err
		}

		var workflowName string

		if flags.workflowName != "" {
			workflowName = flags.workflowName
		} else {
			// If there are multiple workflows defined for the repo, prompt user for which workflow they would like to use
			if repoWorkflows.GetTotalCount() > 1 {
				var workflowNames []string
				for _, workflow := range repoWorkflows.Workflows {
					workflowNames = append(workflowNames, workflow.GetName())
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

				log.Debugf("Selected workflowName: %s", workflowNames[idx])

				workflowName = workflowNames[idx]
			} else {
				// Otherwise if there are only a single workflow, use that one
				workflowName = repoWorkflows.Workflows[0].GetName()
			}
		}

		s := utils.RandomSpinner("Waiting for new workflow run")

		newWorkflow, err := utils.TrackNewWorkflowRun(workflowName, func() {
			git.Push(false)
			s.Start()
		})
		if err != nil {
			log.Fatalf("Failed to track new workflow: %w", err)
		}

		err = utils.OpenInBrowser([]string{newWorkflow.GetHTMLURL()})
		if err != nil {
			log.Fatalf("Failed to open workflow in browser: %w", err)
		}
		s.Stop()

		return nil
	},
}

func init() {
	cmd.RootCmd.AddCommand(pushCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// pushCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// pushCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
	pushCmd.Flags().StringVarP(&flags.workflowName, "workflowName", "w", "", " case insensitive name fo the workflow name to open")
}
