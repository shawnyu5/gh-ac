# Gh actions

- `gh ac force` - get the most recent workflow for current repo. Perform a `git push --force` on the current branch. Then check repo workflow runs every 5s till a new workflow is started. Display workflow URL to user
- `gh ac <push|force> -w <WORKFLOW_NAME>` - specify the workflow name to search for. This is not case sensitive.
  - if there are currently staged changes, warn user before proceeding
- `gh ac <push|force> --url` - print out the workflow URL instead of opening it in browser
- `gh ac config --hostname` to get a custom hostname (for Github enterprise)
- `gh ac <command> -v` - configure logging level. More time the flag is specified, the more granular the logging
- `gh ac push` - push unpushed commits if there are any, and return the workflow url
