package force

import (
	"github.com/charmbracelet/log"
	"github.com/shawnyu5/gh-ac/cmd"
	"github.com/shawnyu5/gh-ac/git"
	"github.com/shawnyu5/gh-ac/utils"
	"github.com/spf13/cobra"
)

type cmdFlags struct {
	// Name of workflowName
	workflowName string
	// Toggle print the URL to workflowName instead of opening it in browser. Defaults to false
	printUrl bool
}

var flags cmdFlags

// forceCmd represents the force command
var forceCmd = &cobra.Command{
	Use:   "force",
	Short: "Force push to the current branch to trigger a workflow run, and open it in the browser",
	Long:  `Runs 'git commit --amend --no-edit && git push --force', and opens the newly triggered workflow run in the browser `,
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

		s := utils.RandomSpinner("Looking for new workflow run")

		newWorkflow, err := utils.TrackNewWorkflowRun(workflowName, func() {
			git.Commit([]string{"--amend", "--no-edit"})
			git.Push(true)
			s.Start()
		})
		if err != nil {
			log.Fatalf("Failed to track new workflow: %s", err)
		}

		s.Stop()
		err = utils.OpenInBrowser([]string{newWorkflow.GetHTMLURL()})
		if err != nil {
			log.Fatalf("Failed to open workflow in browser: %s", err)
		}
	},
}

func init() {
	cmd.RootCmd.AddCommand(forceCmd)
	forceCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
}
