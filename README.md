# Gh actions

- `gh ac commit` - get the most recent workflow for current repo. Run `git commit`, `git push`. Then check repo workflow runs every 5s till a new workflow is started. Display workflow URL to user
  - `gh ac commit -m` - pass the commit message with the command, by passing the need for `git commit`
- `gh ac start` - get the most recent workflow for current repo. Perform a `git push --force` on the current branch. Then check repo workflow runs every 5s till a new workflow is started. Display workflow URL to user
