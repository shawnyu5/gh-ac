// Package gh A wrapper around `gh`, that takes into account configured `hostname`
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
	// Append `--hostname` flag the gh cli command.
	//
	// Some commands such as `gh workflow run` does not support the `--hostname` flag. Default: false
	appendHostName bool
	// Parse the output of the gh command as json. Default: true
	parseOutputJson bool
}

// New create a new instance of Cmd
func New[T any]() *Cmd[T] {
	return &Cmd[T]{
		parseOutputJson: true,
	}
}

// Arg appends an argument to the command
func (c *Cmd[T]) Arg(a string) *Cmd[T] {
	c.args = append(c.args, a)
	return c
}

// AppendHostName appends the host name to the gh cli command.
//
// Default: false
func (c *Cmd[T]) AppendHostName() *Cmd[T] {
	c.appendHostName = true
	return c
}

// ParseOutputJson toggles parsing the output as json. Defaults: true
func (c *Cmd[T]) ParseOutputJson(a bool) *Cmd[T] {
	c.parseOutputJson = a
	return c
}

// Exec executes the gh command with the args, appending the hostname flag if configured
//
// Returns output of the command parsed into `T`
func (c *Cmd[T]) Exec() (output *T, err error) {
	cfg, err := config.Load()
	if err != nil {
		return nil, err
	}
	if c.appendHostName && cfg.HostName != "" {
		c.args = append(c.args, "--hostname", cfg.HostName)
	}

	log.Debugf("Executing command `gh` with arguments %s", c.args)
	stdout, stderr, err := gh.Exec(c.args...)
	if err != nil {
		log.Error(stderr.String())
		return nil, err
	}

	if !c.parseOutputJson {
		return nil, nil
	}

	var jsonResult T
	err = json.Unmarshal(stdout.Bytes(), &jsonResult)
	if err != nil {
		return nil, err
	}
	return &jsonResult, nil
}
