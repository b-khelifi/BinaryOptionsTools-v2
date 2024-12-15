# This is the core for all the `BinaryOptionsTools-v2` versions in the different languages

## Supported Languages (todo)
* Python
* JavaScript
* C / C++
* Java
* Dart

## Todo
* Clean the code and add more logging info

### General
* Make `WebSocketClient` struct more general and create some traits like:
* * `Connect` --> How to connect to websocket
* * `Processor` --> How to process every `tokio_tungstenite::tungstenite::Message`
* * `Sender` --> Struct Or class that will work be shared between threads
* * `Data` --> All the possible data management

### Pocket Option
* Add support for Signals (No clue how to start)
* Add support for pending trades (Seems easy and will add a lot new features to the api)

