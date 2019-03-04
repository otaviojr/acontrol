package main

import "fmt"

type FingerCommandAction struct {
  BaseCommandAction
}

func (action FingerCommandAction) usage() {
  fmt.Println("\tUsage: acontrol-cli finger <command> <options>\r\n")
}

func (action FingerCommandAction) run() bool {
  fmt.Println("FingerPrint command action")
  return false
}
