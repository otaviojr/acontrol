package main

//import "os"
import "fmt"

type BaseCommandAction struct {
  name string
  description string
}

type CommandAction interface {
  getName() string
  getDescription() string
  run() bool
  usage()
}

func (action BaseCommandAction) getName() string {
  return action.name
}

func (action BaseCommandAction) getDescription() string {
  return action.description
}

func (action BaseCommandAction) run() bool {
  fmt.Println("\tRunning run on base class. Something is wrong.\r\n")
  return false
}

func (action BaseCommandAction) usage() {
  fmt.Println("\tRunning usage on base class. Something is wrong.\r\n")
}
