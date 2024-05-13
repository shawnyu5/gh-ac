package gh

import (
	"encoding/json"
	"github.com/charmbracelet/log"
	"github.com/cli/go-gh/v2"
	"github.com/shawnyu5/gh-ac/config"
)

// Cmd a wrapper around gh. `T` is the expected return type of the command
type Cmd[T any] struct {
	// Arguments passed to the command
	args []string
}

// New create a new instance of Cmd
func New[T any]() *Cmd[T] {
	return &Cmd[T]{}
}

// Arg appends an argument to the command
func (c *Cmd[T]) Arg(a string) *Cmd[T] {
	c.args = append(c.args, a)
	return c
}

// Exec executes the gh command with the args, appending the hostname flag if configured
//
// Returns output of the command parsed into `T`
func (c *Cmd[T]) Exec() (output *T, err error) {
	con, err := config.Load()
	if err != nil {
		return nil, err
	}
	if con.HostName != "" {
		c.args = append(c.args, "--hostname", con.HostName)
	}

	// If we dont need to fetch all pages, then execute and return the result
	var jsonResult T
	log.Debugf("Executing command `gh` with arguments %s", c.args)
	stdout, _, err := gh.Exec(c.args...)
	if err != nil {
		return nil, err
	}
	err = json.Unmarshal(stdout.Bytes(), &jsonResult)
	if err != nil {
		return nil, err
	}
	return &jsonResult, nil
}
