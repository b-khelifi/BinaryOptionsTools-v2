from asyncronous import PocketOptionAsync, async_connect
import asyncio

class PocketOption:
    def __init__(self, ssid: str, demo: bool):
        "Creates a new instance of the PocketOption class"
        self.loop = asyncio.new_event_loop()
        self._client: PocketOptionAsync = self.loop.run_until_complete(
            async_connect(ssid, demo)
        )
    
    def __del__(self):
        self.loop.close()

    def buy(self, asset: str, amount: float, time: int, check_win: bool = False):
        """
        Takes the asset, and amount to place a buy trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple with the result of the trade ("win", "loss", "draw") and the trade as a dict
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        future = self.loop.run_in_executor(None, self._client.buy, asset, amount, time, check_win)
        """
        return self.loop.run_until_complete(self._client.buy(asset, amount, time, check_win))
       
    def sell(self, asset: str, amount: float, time: int, check_win: bool = False):
        """
        Takes the asset, and amount to place a sell trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple with the result of the trade ("win", "loss", "draw") and the trade as a dict
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        return self.loop.run_until_complete(self._client.sell(asset, amount, time, check_win))
    
    def check_win(self, id: str):
        """Returns a tuple with the result of the trade ("win", "loss", "draw") and the trade as a dict"""
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
    
    def payout(self, asset: None | str | list[str] = None):
        "Returns a dict of asset | payout for each asset, if 'asset' is not None then it will return the payout of the asset or a list of the payouts for each asset it was passed"
        return self.loop.run_until_complete(self._client.payout(asset))