package dispatch

import (
	"github.com/charmbracelet/log"
	"github.com/shawnyu5/gh-ac/cmd"
	"github.com/shawnyu5/gh-ac/gh"
	"github.com/shawnyu5/gh-ac/git"
	"github.com/shawnyu5/gh-ac/utils"

	"github.com/spf13/cobra"
)

type cmdFlags struct {
	// Name of workflowName
	workflowName string
	// Branch or commit to reference. Defaults to current branch
	githubRef string
	// Input to pass to workflow, in the form KEY=VALUE
	body string
}

var flags cmdFlags

// dispatchCmd represents the dispatch command
var dispatchCmd = &cobra.Command{
	Use:   "dispatch",
	Short: "Create a workflow dispatch event, and open the workflow in the browser",
	Run: func(cmd *cobra.Command, args []string) {
		var workflowName string
		_ = workflowName

		if flags.workflowName != "" {
			workflowName = flags.workflowName
		} else {
			name, err := utils.SelectRepoWorkflowName()
			if err != nil {
				log.Fatalf("Failed to select target workflow: %w", err)
			}
			workflowName = *name
		}

		var githubRef string
		if flags.githubRef != "" {
			githubRef = flags.githubRef
		} else {
			var err error
			githubRef, err = git.CurrentBranchName()
			if err != nil {
				log.Fatalf("Failed to get current branch name: %s", err)
			}
		}

		s := utils.RandomSpinner("Looking for new workflow run\n")

		newWorkflow, err := utils.TrackNewWorkflowRun(workflowName, func() {
			_, err := gh.New[any]().
				Arg("workflow").
				Arg("run").
				Arg(workflowName).
				Arg("--ref").
				Arg(githubRef).
				ParseOutputJson(false).
				Exec()
			if err != nil {
				log.Fatal(err)
			}

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

	},
}

func init() {
	cmd.RootCmd.AddCommand(dispatchCmd)
	dispatchCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
	dispatchCmd.Flags().StringVar(&flags.githubRef, "ref", "", "the branch or tag name which contains the version of the workflow file you'd like to run")
	dispatchCmd.Flags().StringVarP(&flags.body, "form", "f", "", "Input to pass to workflow, in the form KEY=VALUE")
}
