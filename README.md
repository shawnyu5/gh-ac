# Gh actions

- `gh ac force` - get the most recent workflow for current repo. Perform a `git push --force` on the current branch. Then check repo workflow runs every 5s till a new workflow is started. Display workflow URL to user
- `gh ac <push|force|dispatch> -w <WORKFLOW_NAME>` - specify the workflow name to search for. This is not case sensitive.
  - if there are currently staged changes, warn user before proceeding
- `gh ac <push|force> --url` - print out the workflow URL instead of opening it in browser
- `gh ac config --hostname` to get a custom hostname (for Github enterprise)
- `gh ac <command> -v` - configure logging level. More time the flag is specified, the more granular the logging
- `gh ac push` - push unpushed commits if there are any, and return the workflow url
- `gh ac dispatch` - dispatch a workflow dispatch event
  - allow user to select the workflow to dispatch. There is no way to filter `workflow_dispatch` and non dispatch workflows at the moment, so if the user chooses a non `workflow_dispatch` action, then the http request will fail. The error should be displayed to the user
  - allow the user to select a branch from the current repo to run the workflow from. Defaults to the current branch
    - `gh ac dispatch --ref <branch_name>`
  - expect the user to pass in the data for the workflow using `-d key=value`. This will be converted to json, and sent in the request body
  - run the workflow using `gh workflow run "Dispatch hello world" -f name=hello -f word=hi --ref <current_branch>`
