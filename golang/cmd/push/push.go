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
				log.Fatalf("Failed to select target workflow: %w", err)
			}
			workflowName = *name
		}

		s := utils.RandomSpinner("Looking for new workflow run")

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
	pushCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
}
