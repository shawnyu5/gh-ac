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
}

var flags cmdFlags

// pushCmd represents the push command
var pushCmd = &cobra.Command{
	Use:   "push",
	Short: "Push current changes and open the workflow run in browser",
	Long: `Push current changes and open workflow in browser

If no workflow run as been started, this command will wait indefinite until a new workflow run is started`,
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

		s.Stop()
		err = utils.OpenInBrowser([]string{newWorkflow.GetHTMLURL()})
		if err != nil {
			log.Fatalf("Failed to open workflow in browser: %w", err)
		}

		return nil
	},
}

func init() {
	cmd.RootCmd.AddCommand(pushCmd)
	pushCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
}
