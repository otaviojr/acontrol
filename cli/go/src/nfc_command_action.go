package main

import "fmt"

type NfcCommandAction struct {
}

func (action NfcCommandAction) run() bool {
  fmt.Println("NFC command action")
  return false
}
