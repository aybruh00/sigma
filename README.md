# Sigma - An HTTP proxy server for combining bandwidth of two or more internet connections

### Usage
```sigma <listening-port> <iface ip-addresses>```

Currently only IPv4 addressing is supported.

#### Note
Due to the way linux handles ip packet routing, with default settings the program will be able to connect over only one interface. To make it able to connect over other interfaces, read and follow the instructions given in this [stackoverflow answer](https://unix.stackexchange.com/a/752382/580323) to setup your ip tables properly.
This problem is not encountered on windows and the program works out of the box.
