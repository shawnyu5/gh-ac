package config

import (
	"github.com/charmbracelet/log"
	"github.com/shawnyu5/gh-ac/cmd"
	"github.com/shawnyu5/gh-ac/config"
	"github.com/spf13/cobra"
)

type cmdFlags struct {
	hostName string
}

var flags cmdFlags

type Config struct {
	// Custom hostname to run gh cli commands with
	HostName string
}

// configCmd represents the config command
var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Set config values",
	Run: func(cmd *cobra.Command, args []string) {
		err := config.Write(config.Config{
			HostName: flags.hostName,
		})
		log.Infof("Hostname set to %s", flags.hostName)
		if err != nil {
			log.Fatalf("Failed to write to config: %s", err)
		}
	},
}

func init() {
	cmd.RootCmd.AddCommand(configCmd)
	configCmd.Flags().StringVar(&flags.hostName, "hostname", "", "set the hostname")
	configCmd.MarkFlagRequired("hostname")
}
