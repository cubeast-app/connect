# Cubeast Connect

## About

Cubeast Connect is a Bluetooth proxy for Windows, MacOS and Linux. It uses system native Bluetooth API to provide a Websocket API to connect to Bluetooth devices.
This allows Cubeast to connect to Bluetooth devices without having to depend on browser's Web Bluetooth API. This solves 2 major issues with Web Bluetooth API:
* Web Bluetooth API is not supported in all browsers - would allow Cubeast to work in all browsers
* Web Bluetooth feature set is limited - would allow Cubeast to, among others: connect to cubes automatically when the app starts and easily connect to Gan cubes

## API

Currently the app exposes a Websocket API on port 17430. Example of an exchange:

```
-> {"type":"request", "id":"1", "request":{"method":"version"}}
<- {"type":"response","id":"1","response":{"result":"version","version":1}}
-> {"type":"request", "id":"2", "request":{"method":"start-discovery"}}
<- {"type":"response","id":"2","response":{"result":"ok"}}
<- {"type":"broadcast","broadcast":{"name":"discovered-devices","devices":[{"id":"hci0/dev_E7_25_86_0E_40_5B","name":"GAN-ST05B","address":"E7:25:86:0E:40:5B","signal_strength":null,"manufacturer_data":{}}]}}
-> {"type":"request", "id":"3", "request":{"method":"stop-discovery"}}
<- {"type":"response","id":"3","response":{"result":"ok"}}
```
