package config

import (
	"github.com/charmbracelet/log"
	"github.com/spf13/viper"
	"os"
)

type Config struct {
	// Custom hostname to run gh cli commands with
	HostName string
}

// Load loads the config file
func Load() (*Config, error) {
	viper.SetConfigName("gh-ac")
	viper.SetConfigType("yml")
	viper.AddConfigPath("$HOME/.config/gh-ac")
	viper.AddConfigPath(".")
	viper.SetDefault("hostname", "github.com")

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			// Config file not found; ignore error if desired
			log.Debug("No config file found. No hostname configured")
		} else {
			return nil, err
		}
	}

	return &Config{
		HostName: viper.GetString("hostname"),
	}, nil
}

// Write updates the config file
func Write(config Config) error {
	viper.Set("hostname", config.HostName)

	homeDir, err := os.UserHomeDir()
	if err != nil {
		log.Fatal(err)
	}
	os.Mkdir(homeDir+"/.config/gh-ac/", 0777)

	err = viper.WriteConfigAs(homeDir + "/.config/gh-ac/gh-ac.yml")
	return err
}
