from .asyncronous import PocketOptionAsync
from BinaryOptionsToolsV2.validator import Validator
from datetime import timedelta

import asyncio
import json


class SyncSubscription:
    def __init__(self, subscription):
        self.subscription = subscription
        
    def __iter__(self):
        return self
        
    def __next__(self):
        return json.loads(next(self.subscription))        
    

class PocketOption:
    def __init__(self, ssid: str):
        "Creates a new instance of the PocketOption class"
        self.loop = asyncio.new_event_loop()
        self._client = PocketOptionAsync(ssid)
    
    def __del__(self):
        self.loop.close()

    def buy(self, asset: str, amount: float, time: int, check_win: bool = False) -> tuple[str, dict]:
        """
        Takes the asset, and amount to place a buy trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        return self.loop.run_until_complete(self._client.buy(asset, amount, time, check_win))
       
    def sell(self, asset: str, amount: float, time: int, check_win: bool = False) -> tuple[str, dict]:
        """
        Takes the asset, and amount to place a sell trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        return self.loop.run_until_complete(self._client.sell(asset, amount, time, check_win))
    
    def check_win(self, id: str) -> dict:
        """Returns a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)"""
        return self.loop.run_until_complete(self._client.check_win(id))

    def get_candles(self, asset: str, period: int, offset: int) -> list[dict]:
        """
        Takes the asset you want to get the candles and return a list of raw candles in dictionary format
        Each candle contains:
            * time: using the iso format
            * open: open price
            * close: close price
            * high: highest price
            * low: lowest price
        """
        return self.loop.run_until_complete(self._client.get_candles(asset, period, offset))

    def balance(self) -> float:
        "Returns the balance of the account"
        return self.loop.run_until_complete(self._client.balance())
    
    def opened_deals(self) -> list[dict]:
        "Returns a list of all the opened deals as dictionaries"
        return self.loop.run_until_complete(self._client.opened_deals())
    
    def closed_deals(self) -> list[dict]:
        "Returns a list of all the closed deals as dictionaries"
        return self.loop.run_until_complete(self._client.closed_deals())      
    
    def clear_closed_deals(self) -> None:
        "Removes all the closed deals from memory, this function doesn't return anything"
        self.loop.run_until_complete(self._client.clear_closed_deals())
        
    def payout(self, asset: None | str | list[str] = None) -> dict | list[str] | int:
        "Returns a dict of asset | payout for each asset, if 'asset' is not None then it will return the payout of the asset or a list of the payouts for each asset it was passed"
        return self.loop.run_until_complete(self._client.payout(asset))
    
    def history(self, asset: str, period: int) -> list[dict]:
        "Returns a list of dictionaries containing the latest data available for the specified asset starting from 'period', the data is in the same format as the returned data of the 'get_candles' function."
        return self.loop.run_until_complete(self._client.history(asset, period))

    def subscribe_symbol(self, asset: str) -> SyncSubscription:
        """Returns a sync iterator over the associated asset, it will return real time raw candles and will return new candles while the 'PocketOption' class is loaded if the class is droped then the iterator will fail"""
        return SyncSubscription(self.loop.run_until_complete(self._client._subscribe_symbol_inner(asset)))

    def subscribe_symbol_chuncked(self, asset: str, chunck_size: int) -> SyncSubscription:
        """Returns a sync iterator over the associated asset, it will return real time candles formed with the specified amount of raw candles and will return new candles while the 'PocketOption' class is loaded if the class is droped then the iterator will fail"""
        return SyncSubscription(self.loop.run_until_complete(self._client._subscribe_symbol_chuncked_inner(asset, chunck_size)))
    
    def subscribe_symbol_timed(self, asset: str, time: timedelta) -> SyncSubscription:
        """
        Returns a sync iterator over the associated asset, it will return real time candles formed with candles ranging from time `start_time` to `start_time` + `time` allowing users to get the latest candle of `time` duration and will return new candles while the 'PocketOption' class is loaded if the class is droped then the iterator will fail
        Please keep in mind the iterator won't return a new candle exactly each `time` duration, there could be a small delay and imperfect timestamps
        """
        return SyncSubscription(self.loop.run_until_complete(self._client._subscribe_symbol_timed_inner(asset, time)))
    
    def send_raw_message(self, message: str) -> None:
        """
        Sends a raw WebSocket message without waiting for a response.
        
        Args:
            message: Raw WebSocket message to send (e.g., '42["ping"]')
            
        Example:
            ```python
            client = PocketOption(ssid)
            client.send_raw_message('42["ping"]')
            ```
        """
        self.loop.run_until_complete(self._client.send_raw_message(message))
        
    def create_raw_order(self, message: str, validator: Validator) -> str:
        """
        Sends a raw WebSocket message and waits for a validated response.
        
        Args:
            message: Raw WebSocket message to send
            validator: Validator instance to validate the response
            
        Returns:
            str: The first message that matches the validator's conditions
            
        Example:
            ```python
            from BinaryOptionsToolsV2.validator import Validator
            
            client = PocketOption(ssid)
            validator = Validator.starts_with('451-["signals/load"')
            response = client.create_raw_order(
                '42["signals/subscribe"]',
                validator
            )
            ```
        """
        return self.loop.run_until_complete(self._client.create_raw_order(message, validator))
        
    def create_raw_order_with_timout(self, message: str, validator: Validator, timeout: timedelta) -> str:
        """
        Similar to create_raw_order but with a timeout.
        
        Args:
            message: Raw WebSocket message to send
            validator: Validator instance to validate the response
            timeout: Maximum time to wait for a valid response
            
        Returns:
            str: The first message that matches the validator's conditions
            
        Raises:
            TimeoutError: If no valid response is received within the timeout period
            
        Example:
            ```python
            from datetime import timedelta
            from BinaryOptionsToolsV2.validator import Validator
            
            client = PocketOption(ssid)
            validator = Validator.contains('"status":"success"')
            try:
                response = client.create_raw_order_with_timout(
                    '42["trade/start"]',
                    validator,
                    timedelta(seconds=5)
                )
            except TimeoutError:
                print("Operation timed out")
            ```
        """
        return self.loop.run_until_complete(self._client.create_raw_order_with_timeout(message, validator, timeout))
    
    def create_raw_order_with_timeout_and_retry(self, message: str, validator: Validator, timeout: timedelta) -> str:
        """
        Similar to create_raw_order_with_timout but with automatic retry on failure.
        
        Args:
            message: Raw WebSocket message to send
            validator: Validator instance to validate the response
            timeout: Maximum time to wait for each attempt
            
        Returns:
            str: The first message that matches the validator's conditions
            
        Notes:
            - Uses exponential backoff for retries
            - More resilient to temporary network issues
            - Suitable for important operations that must succeed
            
        Example:
            ```python
            from datetime import timedelta
            from BinaryOptionsToolsV2.validator import Validator
            
            client = PocketOption(ssid)
            validator = Validator.all([
                Validator.contains('"type":"trade"'),
                Validator.contains('"status":"completed"')
            ]).raw_validator
            
            response = client.create_raw_order_with_timeout_and_retry(
                '42["trade/execute"]',
                validator,
                timedelta(seconds=10)
            )
            ```
        """
        return self.loop.run_until_complete(self._client.create_raw_order_with_timeout_and_retry(message, validator, timeout))
 
    def create_raw_iterator(self, message: str, validator: Validator, timeout: timedelta | None = None) -> SyncSubscription:
        """
        Creates a synchronous iterator that yields validated WebSocket messages.
        
        Args:
            message: Initial WebSocket message to send
            validator: Validator instance to filter incoming messages
            timeout: Optional timeout for the entire stream
            
        Returns:
            SyncSubscription yielding validated messages
            
        Example:
            ```python
            from datetime import timedelta
            from BinaryOptionsToolsV2.validator import Validator
            
            client = PocketOption(ssid)
            # Create validator for price updates
            validator = Validator.regex(r'{"price":\d+\.\d+}').raw_validator
            
            # Subscribe to price stream
            stream = client.create_raw_iterator(
                '42["price/subscribe"]',
                validator,
                timeout=timedelta(minutes=5)
            )
            
            # Process price updates
            for message in stream:
                price_data = json.loads(message)
                print(f"Current price: {price_data['price']}")
            ```
            
        Notes:
            - The iterator will continue until the timeout is reached or an error occurs
            - If timeout is None, the iterator will continue indefinitely
            - The stream can be stopped by breaking out of the loop
        """
        return SyncSubscription(self.loop.run_until_complete(self._client.create_raw_iterator(message, validator, timeout)))

    def get_server_time(self) -> int:
        """Returns the current server time as a UNIX timestamp"""
        return self.loop.run_until_complete(self._client.get_server_time())

    def is_demo(self) -> bool:
        """
        Checks if the current account is a demo account.

        Returns:
            bool: True if using a demo account, False if using a real account

        Examples:
            ```python
            # Basic account type check
            client = PocketOption(ssid)
            is_demo = client.is_demo()
            print("Using", "demo" if is_demo else "real", "account")

            # Example with balance check
            def check_account():
                is_demo = client.is_demo()
                balance = client.balance()
                print(f"{'Demo' if is_demo else 'Real'} account balance: {balance}")

            # Example with trade validation
            def safe_trade(asset: str, amount: float, duration: int):
                is_demo = client.is_demo()
                if not is_demo and amount > 100:
                    raise ValueError("Large trades should be tested in demo first")
                return client.buy(asset, amount, duration)
            ```
        """
        return self.loop.run_until_complete(self._client.is_demo())
