package push

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

// pushCmd represents the push command
var pushCmd = &cobra.Command{
	Use:   "push",
	Short: "Push current changes and open workflowName in browser",
	Long: `Push current changes and open workflowName in browser

If no workflowName run as been started, this command will wait indefinite until a new workflowName run is started`,
	RunE: func(cmd *cobra.Command, args []string) error {
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

		s := utils.RandomSpinner("Looking for new workflow run\n")

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
	pushCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
}
