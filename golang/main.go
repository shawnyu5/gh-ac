package main

import (
	"github.com/shawnyu5/gh-ac/cmd"
	_ "github.com/shawnyu5/gh-ac/cmd/config"
	_ "github.com/shawnyu5/gh-ac/cmd/push"
)

func main() {
	cmd.Execute()

	//fmt.Println("hi world, this is the gh-ac extension!")
	//client, err := api.DefaultRESTClient()
	//if err != nil {
	//	fmt.Println(err)
	//	return
	//}
	//response := struct{ Login string }{}
	//err = client.Get("user", &response)
	//if err != nil {
	//	fmt.Println(err)
	//	return
	//}
	//fmt.Printf("running as %s\n", response.Login)
}

// For more examples of using go-gh, see:
// https://github.com/cli/go-gh/blob/trunk/example_gh_test.go
