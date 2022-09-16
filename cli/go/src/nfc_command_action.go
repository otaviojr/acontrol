package main

import "os"
import "fmt"

/*
 * NFC main context action
 */
type NfcCommandAction struct {
  BaseCommandAction
}

/*
 * NFC commands actions
 */
type NfcAuthorizeCommandAction struct {
  BaseCommandAction
}

type NfcRestoreCommandAction struct {
  BaseCommandAction
}

type NfcListCommandAction struct {
  BaseCommandAction
}

type NfcDeleteCommandAction struct {
  BaseCommandAction
}

/*
 * All commands supported in this context
 */
var commands []Option = []Option {
  Option {
    option: "authorize",
    action: NfcAuthorizeCommandAction{BaseCommandAction{name: "Authorize", description: "Authorize a new card to access this device",},},
  },
  Option {
    option: "restore",
    action: NfcRestoreCommandAction{BaseCommandAction{name: "Restore", description: "Restore a formatted card to factory state",},},
  },
  Option {
    option: "list",
    action: NfcListCommandAction{BaseCommandAction{name: "List", description: "List all registered cards",},},
  },
  Option {
    option: "delete",
    action: NfcDeleteCommandAction{BaseCommandAction{name: "Delete", description: "Unauthorize a card to access this device.",},},
  },
}

/*
 * NFC main context implementation
 */
func (action NfcCommandAction) usage() {
  fmt.Println("\tUsage: acontrol-cli nfc <command> <option>\r\n")

  fmt.Println("\tListing all commands:\r\n")
  for _, command := range commands {
    fmt.Printf("\tCommand %s\r\n\t%s\r\n\r\n", command.action.getName(), command.action.getDescription())
    command.action.usage()
  }
}

func (action NfcCommandAction) run() bool {
  if len(os.Args) < 3 {
    fmt.Println("NFC command: missing argument. Type acontrol-cli help to usage options")
    os.Exit(2)
  }

  cmdPtr := &os.Args[2]

  var current_command *Option = nil

  for _, command := range commands {
    if command.option == *cmdPtr {
      current_command = &command
      break
    }
  }

  if current_command == nil {
    fmt.Printf("Command %s not found. Type acontrol-cli help to usage options.\r\n\r\n", *cmdPtr)
    os.Exit(2)
  }

  return current_command.action.run()
}

/*
 * NFC commands implementation
 */
func (action NfcAuthorizeCommandAction) usage() {
  fmt.Println("\t\tUsage: acontrol-cli nfc authorize <options>\r\n")
}

func (action NfcAuthorizeCommandAction) run() bool {

  cmdLine := createCommandLineParser()

  name :=  cmdLine.getStringParameter("name")

  if name == "" {
    fmt.Println("nfc authorize missing arguments. Type acontrol-cli help to usage options.")
    return false
  }

  fmt.Printf("Authorizing %s...\r\n\r\n", name)

  //TODO: Start authorization process via rest service

  return true
}

func (action NfcRestoreCommandAction) usage() {
  fmt.Println("\t\tUsage: acontrol-cli nfc restore <options>\r\n")
}

func (action NfcRestoreCommandAction) run() bool {
  return false
}

func (action NfcListCommandAction) usage() {
  fmt.Println("\t\tUsage: acontrol-cli nfc list <options>\r\n")
}

func (action NfcListCommandAction) run() bool {
  return false
}

func (action NfcDeleteCommandAction) usage() {
  fmt.Println("\t\tUsage: acontrol-cli nfc delete <options>\r\n")
}

func (action NfcDeleteCommandAction) run() bool {
  return false;
}
