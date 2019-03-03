package main

import "os"
//import "flag"
import "fmt"

type Context struct {
  context string
  action CommandAction
}

var contexts []Context = []Context {
  Context {
    context: "nfc",
    action: NfcCommandAction{},
  },
  Context {
    context: "finger",
    action: FingerCommandAction{},
  },
}

func init() {
}

func usage() {
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
    fmt.Println("Context not informed. Use -help to show help.")
    os.Exit(1)
  }

  var current_context *Context = nil

  for _, context := range contexts {
    if context.context == *contextPtr {
      current_context = &context
      break
    }
  }

  if current_context == nil {
    fmt.Printf("Context %s not fount. Type acontrol-cli help to show usage.\r\n", *contextPtr)
    os.Exit(1)
  }
  //cmdPtr := flag.String("command", "", "Command to be executed")

  current_context.action.run();

  //flag.Parse()
}
