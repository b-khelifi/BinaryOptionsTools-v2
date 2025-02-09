👉 [Join us on Discord](https://discord.gg/T3FGXcmd)
# BinaryOptionsToolsV2
Path to the github repository for this version of the project: https://github.com/ChipaDevTeam/BinaryOptionsTools-v2/commit/5ef8aa83bcf3f9b8a55f0d9d5e69a1a48397ed7f

## How to install
Install it with PyPi using the following command:
```bash
pip install binaryoptionstoolsv2==0.1.6a2
```

## Supported Os
Currently only suppot for Windows is available.

## Supported Python versions
Currently only python 3.9 to 3.12 is suported.

## Compile from souce (Not recommended)
* Make sure you have `rust` and `cargo` installed ([Check here](https://www.rust-lang.org/tools/install))

* Install `[maturin](https://www.maturin.rs/installation)` in order to compile the library

* Once the source dowloaded (using `git clone https://github.com/ChipaDevTeam/BinaryOptionsTools-v2.git`) execute the following commands:
To create the `.whl` file
```bash
// Inside the root folder
cd BinaryOptionsToolsV2
maturin build -r 

// Once the command is executed it should print a path to a .whl file, copy it and then run
pip install path/to/file.whl
```

To install the library in a local virtual environment
```bash
// Inside the root folder
cd BinaryOptionsToolsV2

// Activate the virtual environment if not done alreade 

// Execute the following command and it shoult automatically install the library in the VM
maturin develop
```

## Docs
Comprehensive Documentation for BinaryOptionsToolsV2 

1. `__init__.py` 

This file initializes the Python module and organizes the imports for both synchronous and asynchronous functionality. 

Key Details 

- **Imports `BinaryOptionsToolsV2`**: Imports all elements and documentation from the Rust module. 
- **Includes Submodules**: Imports and exposes `pocketoption` and `tracing` modules for user convenience. 

Purpose 

Serves as the entry point for the package, exposing all essential components of the library. 

### Inside the `pocketoption` folder there are 2 main files
2. `asyncronous.py` 

This file implements the `PocketOptionAsync` class, which provides an asynchronous interface to interact with Pocket Option. 

Key Features of PocketOptionAsync 

- **Trade Operations**: 
  - `buy()`: Places a buy trade asynchronously. 
  - `sell()`: Places a sell trade asynchronously. 
  - `check_win()`: Checks the outcome of a trade ('win', 'draw', or 'loss'). 
- **Market Data**: 
  - `get_candles()`: Fetches historical candle data. 
  - `history()`: Retrieves recent data for a specific asset. 
- **Account Management**: 
  - `balance()`: Returns the current account balance. 
  - `opened_deals()`: Lists all open trades. 
  - `closed_deals()`: Lists all closed trades. 
  - `payout()`: Returns payout percentages. 
- **Real-Time Data**: 
  - `subscribe_symbol()`: Provides an asynchronous iterator for real-time candle updates. 

Helper Class - `AsyncSubscription` 

Facilitates asynchronous iteration over live data streams, enabling non-blocking operations. 

Example Usage 

```python
from BinaryOptionsToolsV2.pocketoption import PocketOptionAsync 
import asyncio 
 
async def main(): 
    client = PocketOptionAsync(ssid="your-session-id") 
    await asyncio.sleep(5)
    balance = await client.balance() 
    print("Account Balance:", balance) 
 
asyncio.run(main()) 

``` 

3. `syncronous.py` 

This file implements the `PocketOption` class, a synchronous wrapper around the asynchronous interface provided by `PocketOptionAsync`. 

Key Features of PocketOption 

- **Trade Operations**: 
  - `buy()`: Places a buy trade using synchronous execution. 
  - `sell()`: Places a sell trade. 
  - `check_win()`: Checks the trade outcome synchronously. 
- **Market Data**: 
  - `get_candles()`: Fetches historical candle data. 
  - `history()`: Retrieves recent data for a specific asset. 
- **Account Management**: 
  - `balance()`: Retrieves account balance. 
  - `opened_deals()`: Lists all open trades. 
  - `closed_deals()`: Lists all closed trades. 
  - `payout()`: Returns payout percentages. 
- **Real-Time Data**: 
  - `subscribe_symbol()`: Provides a synchronous iterator for live data updates. 

Helper Class - `SyncSubscription` 

Allows synchronous iteration over real-time data streams for compatibility with simpler scripts. 

Example Usage 

 
```python
from BinaryOptionsToolsV2.pocketoption import PocketOption 
import time

client = PocketOption(ssid="your-session-id") 
time.sleep(5)
balance = client.balance() 
print("Account Balance:", balance) 

```
 
 

4. Differences Between PocketOption and PocketOptionAsync 

| Feature                | PocketOption (Synchronous) | PocketOptionAsync (Asynchronous) | 
|------------------------|----------------------------|----------------------------------| 
| **Execution Type**     | Blocking                  | Non-blocking                    | 
| **Use Case**           | Simpler scripts           | High-frequency or real-time tasks | 
| **Performance**        | Slower for concurrent tasks | Scales well with concurrent operations | 