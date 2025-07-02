# Cubeast Connect

## About

Cubeast Connect is a Bluetooth proxy for Windows, MacOS and Linux. It uses system native Bluetooth API to provide a Websocket API to connect to Bluetooth devices.
This allows Cubeast to connect to Bluetooth devices without having to depend on browser's Web Bluetooth API. This solves 2 major issues with Web Bluetooth API:
* Web Bluetooth API is not supported in all browsers - would allow Cubeast to work in all browsers
* Web Bluetooth feature set is limited - would allow Cubeast to, among others: connect to cubes automatically when the app starts and easily connect to Gan cubes

## API

Currently the app exposes a Websocket API on port 17430. Example of an exchange:

```
-> {"type":"request", "id":"1", "request": {"type":"version"}}
<- {"type":"response", "id":"1", "response": {"result":"version", "version":1}}
-> {"type":"request", "id":"2", "request":{"type":"start-discovery"}}
<- {"type":"response","id":"2","response":{"result":"ok"}}
<- {"type": "discovered-devices","devices": [{
      "id": "hci0/dev_70_19_88_8F_9F_CB",
      "name": "GANicE2_9FCB",
      "address": "70:19:88:8F:9F:CB",
      "signal_strength": -60,
      "manufacturer_data": {
        "1": [
          0,
          0,
          0,
          203,
          159,
          143,
          136,
          25,
          112
        ]
      }
    },
    ]}
-> {"type":"request", "id":"3", "request":{"type":"stop-discovery"}}
<- {"type":"response","id":"3","response":{"result":"ok"}}
-> {"type":"request", "id":"4", "request":{"type":"write-characteristic", "device_name": "GANicE2_9FCB", "characteristic_id": "0000fff5-0000-1000-8000-00805f9b34fb", "value":[210, 13, 5, 57, 119, 0, 0, 1, 35, 69, 103, 137, 171, 0, 0, 0]}}
<- {"type":"response", "id":"4", "response":{"result":"ok"}}
-> {"type":"request", "id":"5", "request":{"type":"subscribe-to-characteristic", "device_name": "GANicE2_9FCB", "characteristic_id": "0000fff6-0000-1000-8000-00805f9b34fb"}}
-> {"type":"request", "id":"6", "request":{"type":"unsubscribe-from-characteristic", "device_name": "GANicE2_9FCB", "characteristic_id": "0000fff6-0000-1000-8000-00805f9b34fb"}}
```
