👉 [Join us on Discord](https://discord.gg/T3FGXcmd)
# BinaryOptionsToolsV2

Comprehensive Documentation for BinaryOptionsToolsV2 

1. `__init__.py` 

This file initializes the Python module and organizes the imports for both synchronous and asynchronous functionality. 

Key Details 

- **Imports `BinaryOptionsToolsV2`**: Imports all elements and documentation from the Rust module. 
- **Includes Submodules**: Imports and exposes `asyncronous` and `syncronous` modules for user convenience. 

Purpose 

Serves as the entry point for the package, exposing all essential components of the library. 

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
from BinaryOptionsToolsV2.asyncronous import PocketOptionAsync 
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
from BinaryOptionsToolsV2.syncronous import PocketOption 
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