# This is the core for all the `BinaryOptionsTools-v2` versions in the different languages

## Supported Languages (todo)
* Python
* JavaScript
* C / C++
* Java
* Dart

## Todo
* Clean the code and add more logging info
* Add support for testing for multiple different connections, like passing an iterable
* Add error handling in case there is an error parsing some data, to return an error and not keep waiting (It is for the `send_message` function)
* Add support for pending requests by `time` and by `price`

### General
* Make `WebSocketClient` struct more general and create some traits like:
  * `Connect` --> How to connect to websocket
  * `Processor` --> How to process every `tokio_tungstenite::tungstenite::Message`
  * `Sender` --> Struct Or class that will work be shared between threads
  * `Data` --> All the possible data management

### Pocket Option
* Add support for Signals (No clue how to start)
* Add support for pending trades (Seems easy and will add a lot new features to the api)

### Important
* **Pocket Option** server works on the timezone: `UTC +2`