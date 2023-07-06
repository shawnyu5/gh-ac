# Gh actions

- `gh ac force` - get the most recent workflow for current repo. Perform a `git push --force` on the current branch. Then check repo workflow runs every 5s till a new workflow is started. Display workflow URL to user
- `gh ac <push|force|dispatch> -w <WORKFLOW_NAME>` - specify the workflow name to search for. This is not case sensitive.
  - if there are currently staged changes, warn user before proceeding
- `gh ac <push|force|dispatch> --url` - print out the workflow URL instead of opening it in browser
- `gh ac config --hostname` to get a custom hostname (for Github enterprise)
- `gh ac <command> -v` - configure logging level. More time the flag is specified, the more granular the logging
- `gh ac push` - push unpushed commits if there are any, and return the workflow url
- `gh ac dispatch` - create a workflow dispatch event
  - `gh ac dispatch --ref <branch_name>` to select the branch to dispatch the workflow from. Defaults to the current branch
  - input can be passed to the workflow in the form `gh ac dispatch -d key=val -d key2=val2`
- `gh ac cleanup` - cleanup old workflow that has been renamed. Old workflows are identified by workflows that has the same name as the path to the workflow file.
  - For example, a workflow called `.github/workflows/ci.yml`, whose path is `.github/workflows/ci.yml` will be considered an old workflow, which will be cleaned up.
