package main

import "os"
import "strings"
import "fmt"

type CommandLine struct {
}

func createCommandLineParser() *CommandLine {
  cmdLine := CommandLine {}
  return &cmdLine
}

func (cmdLine CommandLine) findToken(token string) (int, string) {
  position := 0
  ret := ""

  for index, value := range os.Args {
    if strings.HasPrefix(value,fmt.Sprintf("-%s=",token)) {
      position = index
      ret = strings.Split(value,"=")[1]
      break
    } else if value == fmt.Sprintf("--%s", token) {
      position = index
      if len(os.Args) > index+1 {
        argument := os.Args[index + 1]
        if !strings.HasPrefix(argument,"-") && !strings.HasPrefix(argument,"--") {
          ret = os.Args[index + 1]
        }
      }
      break
    }
  }
  return position,ret
}

func (cmdLine CommandLine) getStringParameter(name string) string {
  _,value := cmdLine.findToken(name)
  return value
}
