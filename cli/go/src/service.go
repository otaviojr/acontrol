package main

import "net/http"
import "encoding/json"
//import "io/ioutil"
import "bytes"
import "fmt"

type NfcCard struct {
  id int
  uuid string
  name string
}

type RestService struct {
  hostname string
  port int
  protocol string
  api_key string
}

func createRestService(hostname string, port int) *RestService {
  service := RestService {hostname: hostname, port: port, protocol: "http",}
  return &service
}

func (service RestService) createUri(path string) string {
  return fmt.Sprintf("%s://%s:%d/%s", service.protocol, service.hostname, service.port, path)
}

func (service RestService) nfcListCards() ([]NfcCard,error) {
  resp, err := http.Get(service.createUri("nfc/card"))

  if err != nil {
    return nil, err
  }

  defer resp.Body.Close()

  var obj map[string] interface{}
  decoder := json.NewDecoder(resp.Body)
  err = decoder.Decode(&obj)

  if err != nil {
    return nil, err
  }

  var ret []NfcCard = []NfcCard {}

  cards, ok := obj["cards"].([]interface{})

  if ok {
    for _,value := range cards {

      val, ok := value.(map[string] interface{})

      if ok {
        id, ok := val["id"].(float64)
        if !ok { id = 0 }
        uuid, ok := val["uuid"].(string)
        if !ok { uuid = "" }
        name, ok := val["name"].(string)
        if !ok { name = "" }

        ret = append(ret, NfcCard {
          id: int(id),
          uuid: uuid,
          name: name,
        })
      }
    }
  }

  return ret, nil
}

func (service RestService) nfcAuthorizeCard(card NfcCard) error {

  buf, err := json.Marshal(card)

  if err != nil {
    return err
  }

  resp, err := http.Post(service.createUri("nfc/card/authorize"), "application/json", bytes.NewBuffer(buf))

  if err != nil {
    return err
  }

  defer resp.Body.Close()

  var obj map[string] interface {}
  decoder := json.NewDecoder(resp.Body)
  decoder.Decode(&obj)

  status, ok := obj["status"].(bool)

  if !ok || !status {
    return nil
  }

  return nil
}
