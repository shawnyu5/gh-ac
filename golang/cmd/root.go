package cmd

import (
	"fmt"
	"github.com/charmbracelet/log"
	"github.com/spf13/cobra"
	"os"
)

type cmdFlags struct {
	// Toggle debug logging. Defaults to false
	debug bool
}

var flags cmdFlags

var RootCmd = &cobra.Command{
	Use:   "ac",
	Short: "Fire off Github action workflow runs, and open it in the browser",
	PersistentPreRun: func(cmd *cobra.Command, args []string) {
		if flags.debug {
			log.SetLevel(log.DebugLevel)
		}
	},
	//Run: func(cmd *cobra.Command, args []string) {
	//	fmt.Println("HEYYY")
	//},
}

func init() {
	RootCmd.PersistentFlags().BoolVar(&flags.debug, "debug", false, "toggle debug logging")
}

func Execute() {
	if err := RootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
