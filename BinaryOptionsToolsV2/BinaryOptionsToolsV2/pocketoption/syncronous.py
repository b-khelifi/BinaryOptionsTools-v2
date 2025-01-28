from .asyncronous import PocketOptionAsync
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