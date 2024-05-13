package git

import (
	"fmt"
	"os/exec"
)

// CommitAdmendNoEdit executes `git commit --amend --no-edit`
func CommitAdmendNoEdit() error {
	args := []string{"commit", "--amend", "--no-edit"}
	output, err := exec.Command("git", args...).CombinedOutput()
	fmt.Println(string(output))
	return err
}

// Push executes `git push`. Will force push if `force` is true
func Push(force bool) error {
	// TODO: consider making git push fail when there are nothing to push
	args := []string{"push"}
	if force {
		args = append(args, "--force")
	}
	output, err := exec.Command("git", args...).CombinedOutput()
	fmt.Println(string(output))
	return err
}

// Commit runs `git commit` with `args` passed to it
func Commit(args []string) error {
	cmdArgs := []string{"commit"}
	cmdArgs = append(cmdArgs, args...)
	output, err := exec.Command("git", cmdArgs...).CombinedOutput()
	fmt.Println(string(output))
	return err
}
