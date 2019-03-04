package main

import "os"
//import "flag"
import "fmt"

type HelpCommandAction struct {
  BaseCommandAction
}

var contexts []Option = []Option {
  Option {
    option: "help",
    action: HelpCommandAction{BaseCommandAction {name: "Help", description: "Show usage options",},},
  },
  Option {
    option: "nfc",
    action: NfcCommandAction{BaseCommandAction {name: "NFC", description: "Handle NFC device"},},
  },
  Option {
    option: "finger",
    action: FingerCommandAction{BaseCommandAction {name: "FingerPrint", description: "Handle FingerPrint device"},},
  },
}

func init() {
}

func usage() {
}

func (action HelpCommandAction) usage() {
  fmt.Println("\tUsage: acontrol-cli help\r\n")
}

func (action HelpCommandAction) run() bool {
  fmt.Println("Usage: acontrol-cli <context> <command> <options>\r\n")
  fmt.Println("Listing all contexts: \r\n")
  for _, context := range contexts {
    fmt.Printf("Context %s (%s)\r\n\r\n", context.action.getName(), context.action.getDescription())
    context.action.usage()
  }

  return false;
}

func main() {

  fmt.Println("acontrol - Access Control System")
  fmt.Println("Otavio Ribeiro <otavio.ribeiro@gmail.com>\r\n")

  if len(os.Args) < 2 {
    fmt.Println("Missing arguments. Type acontrol-cli help to show usage options.")
    os.Exit(1)
  }

  contextPtr := &os.Args[1]

  if *contextPtr == "" {
    fmt.Println("Context not informed. Type acontrol-cli help to show usage options.")
    os.Exit(1)
  }

  var current_context *Option = nil

  for _, context := range contexts {
    if context.option == *contextPtr {
      current_context = &context
      break
    }
  }

  if current_context == nil {
    fmt.Printf("Context %s not fount. Type acontrol-cli help to show usage.\r\n", *contextPtr)
    os.Exit(1)
  }
  //cmdPtr := flag.String("command", "", "Command to be executed")

  current_context.action.run()

  //flag.Parse()
}
