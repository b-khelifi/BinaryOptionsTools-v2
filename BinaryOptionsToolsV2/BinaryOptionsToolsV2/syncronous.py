import json
from BinaryOptionsToolsV2.asyncronous import PocketOptionAsync
import asyncio

class PocketOption:
    def __init__(self, ssid: str):
        "Creates a new instance of the PocketOption class"
        self.loop = asyncio.new_event_loop()
        self._client = PocketOptionAsync(ssid)
    
    def __del__(self):
        self.loop.close()

    def buy(self, asset: str, amount: float, time: int, check_win: bool = False):
        """
        Takes the asset, and amount to place a buy trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        return self.loop.run_until_complete(self._client.buy(asset, amount, time, check_win))
       
    def sell(self, asset: str, amount: float, time: int, check_win: bool = False):
        """
        Takes the asset, and amount to place a sell trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        return self.loop.run_until_complete(self._client.sell(asset, amount, time, check_win))
    
    def check_win(self, id: str):
        """Returns a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)"""
        return self.loop.run_in_executor(self._client.check_win(id))

    def get_candles(self, asset: str, period: int, offset: int):
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

    def balance(self):
        "Returns the balance of the account"
        return self.loop.run_until_complete(self._client.balance())
    
    def opened_deals(self):
        "Returns a list of all the opened deals as dictionaries"
        return self.loop.run_until_complete(self._client.opened_deals())
    
    def closed_deals(self):
        "Returns a list of all the closed deals as dictionaries"
        return self.loop.run_until_complete(self._client.closed_deals())      
    
    def clear_closed_deals(self):
        "Removes all the closed deals from memory, this function doesn't return anything"
        self.loop.run_until_complete(self._client.clear_closed_deals())
        
    def payout(self, asset: None | str | list[str] = None):
        "Returns a dict of asset | payout for each asset, if 'asset' is not None then it will return the payout of the asset or a list of the payouts for each asset it was passed"
        return self.loop.run_until_complete(self._client.payout(asset))
    
    def history(self, asset: str, period: int):
        "Returns a list of dictionaries containing the latest data available for the specified asset starting from 'period', the data is in the same format as the returned data of the 'get_candles' function."
        return self.loop.run_until_complete(self._client.history(asset, period))

    def subscribe_symbol(self, asset: str):
        """Returns an async iterator over the associated asset, it will return real time raw candles and will return new candles while the 'PocketOptionAsync' class is loaded if the class is droped then the iterator will fail"""
        return SyncSubscription(self.loop.run_until_complete(self._client._subscribe_symbol_inner(asset)))

class SyncSubscription:
    def __init__(self, subscription):
        self.subscription = subscription
        
    def __iter__(self):
        return self
        
    def __next__(self):
        return json.loads(next(self.subscription))        