# Gh ac

Trigger Github action runs, and open the action run in the browser.

<!-- vim-markdown-toc GFM -->

* [Installation](#installation)
* [Setup](#setup)
* [Usage](#usage)
* [Achieved workflow](#achieved-workflow)
    * [Commit + push](#commit--push)
    * [Amend current commit + force push](#amend-current-commit--force-push)
    * [Workflows with manual triggers](#workflows-with-manual-triggers)
* [Troubleshooting](#troubleshooting)
    * [Expected workflow are not showing up when running push or force](#expected-workflow-are-not-showing-up-when-running-push-or-force)

<!-- vim-markdown-toc -->

## Installation

```bash
gh extension install https://github.com/shawnyu5/gh-ac
```

## Setup

This extension uses the `$BROWSER` environment variable to determine the browser to use. Add the following to your `bashrc` / `zshrc`

```bash
export BROWSER="path/to/browser"
```

## Usage

```text
Fire off Github action workflow runs, and open it in the browser

Usage:
  ac [command]

Available Commands:
  cleanup     Clean up workflow run history for a specific workflow
  config      Set config values
  dispatch    Create a workflow dispatch event, and open the workflow in the browser
  force       Force push to the current branch to trigger a workflow run, and open it in the browser
  help        Help about any command
  push        Push current changes and open the workflow run in browser

Flags:
      --debug   toggle debug logging
  -h, --help    help for ac

Use "ac [command] --help" for more information about a command.
```

## Achieved workflow

This plugin allows the following workflow when developing Github actions:

### Commit + push

This is a typical git workflow you'd follow when developing actions

1. Commit your local changes
2. Use `gh ac push` to push your changes, and select a workflow run to open in the browser

### Amend current commit + force push

Sometimes, you'd want to make a very small change in a workflow, that does not constitute another commit called `testing`. You would like to just bundle your current changes with the previous commit:

1. Stage the changes you would like to push, `git add my-file.txt`
2. Use `gh ac force`, which will run `git commit --amend --no-edit && git push` under the hood, to add your current changes to the previous commit, and force push
3. Select the workflow run to open in the browser

### Workflows with manual triggers

For workflow with `workflow_dispatch` events, this plugin supports triggering a `workflow_dispatch` event, and opening the workflow in the browser.

- Use `gh ac dispatch`, and select a workflow name to send a `workflow_dispatch` event to. It is the user's responsibility to select the workflow with `workflow_dispatch` trigger.
- To send workflow inputs, use `gh ac dispatch -w <workflow name> -f key=value` to pass form body
- To run the workflow on a different branch, pass the `--ref <github ref>` flag to use the workflow defined at the specific `ref`

All above commands supports the `-w` flag, that allows passing the target workflow name as an argument, rather than being prompted for it on every run

## Troubleshooting

### Expected workflow are not showing up when running push or force

If the repo is a forked repo, then you must set the default repo using `gh repo set-default`.
