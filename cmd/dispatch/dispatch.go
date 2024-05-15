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
	body []string
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
				log.Fatalf("Failed to select target workflow: %newWorkflowSpinner", err)
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
				log.Fatalf("Failed to get current branch name: %newWorkflowSpinner", err)
			}
		}

		newWorkflowSpinner := utils.RandomSpinner("Looking for new workflow run")

		newWorkflow, err := utils.TrackNewWorkflowRun(workflowName, func() {
			workflowDispatchSpinner := utils.RandomSpinner("Creating workflow dispatch event")
			workflowDispatchSpinner.Start()
			ghBuilder := gh.New[any]().
				Arg("workflow").
				Arg("run").
				Arg(workflowName).
				Arg("--ref").
				Arg(githubRef).
				ParseOutputJson(false)

			if len(flags.body) != 0 {
				for _, body := range flags.body {
					ghBuilder.
						Arg("-f").
						Arg(body)
				}
			}

			_, err := ghBuilder.Exec()
			if err != nil {
				log.Fatal(err)
			}
			workflowDispatchSpinner.Stop()
			newWorkflowSpinner.Start()
		})
		if err != nil {
			log.Fatalf("Failed to track new workflow: %w", err)
		}

		err = utils.OpenInBrowser([]string{newWorkflow.GetHTMLURL()})
		if err != nil {
			log.Fatalf("Failed to open workflow in browser: %w", err)
		}
		newWorkflowSpinner.Stop()

	},
}

func init() {
	cmd.RootCmd.AddCommand(dispatchCmd)
	dispatchCmd.Flags().StringVarP(&flags.workflowName, "workflow", "w", "", "case insensitive name for the workflow name track")
	dispatchCmd.Flags().StringVar(&flags.githubRef, "ref", "", "the branch or tag name which contains the version of the workflow file you'd like to run")
	dispatchCmd.Flags().StringArrayVarP(&flags.body, "form", "f", []string{}, "Input to pass to workflow, in the form KEY=VALUE")
}
