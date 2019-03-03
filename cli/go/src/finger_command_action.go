package main

import "fmt"

type FingerCommandAction struct {
}

func (action FingerCommandAction) run() bool {
  fmt.Println("FingerPrint command action")
  return false
}
