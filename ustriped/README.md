# ustriped

## Synopsis

* Opens SPI device.
* First argument is the led number
* Waits for [OPC](http://openpixelcontrol.org/) packets over *UDP*
* Writes them to SPI for WS2801 LED controllers, then waits one 1ms
* Additionally supports multiple priorities with a quick timeout

## Build

```
make
```
