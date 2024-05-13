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
