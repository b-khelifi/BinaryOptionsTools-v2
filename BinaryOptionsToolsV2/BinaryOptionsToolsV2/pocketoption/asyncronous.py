from BinaryOptionsToolsV2 import RawPocketOption, Logger
from datetime import timedelta

import asyncio
import json
import time 
import sys 


class AsyncSubscription:
    def __init__(self, subscription):
        self.subscription = subscription
        
    def __aiter__(self):
        return self
        
    async def __anext__(self):
        return json.loads(await anext(self.subscription))
    
# This file contains all the async code for the PocketOption Module
class PocketOptionAsync:
    def __init__(self, ssid: str):
        self.client = RawPocketOption(ssid)
        self.logger = Logger()
        
    
    async def buy(self, asset: str, amount: float, time: int, check_win: bool = False) -> tuple[str, dict]:
        """
        Takes the asset, and amount to place a buy trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        (trade_id, trade) = await self.client.buy(asset, amount, time)
        if check_win:
            return trade_id, await self.check_win(trade_id) 
        else:
            trade = json.loads(trade)
            return trade_id, trade 
       
    async def sell(self, asset: str, amount: float, time: int, check_win: bool = False) -> tuple[str, dict]:
        """
        Takes the asset, and amount to place a sell trade that will expire in time (in seconds).
        If check_win is True then the function will return a tuple containing the trade id and a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)
        If check_win is False then the function will return a tuple with the id of the trade and the trade as a dict
        """
        (trade_id, trade) = await self.client.sell(asset, amount, time)
        if check_win:
            return trade_id, await self.check_win(trade_id)   
        else:
            trade = json.loads(trade)
            return trade_id, trade 
 
    async def check_win(self, id: str) -> dict:
        """Returns a dictionary containing the trade data and the result of the trade ("win", "draw", "loss)"""
        end_time = await self.client.get_deal_end_time(id)
        
        if end_time is not None:
            duration = end_time - int(time.time())
            if duration <= 0:
                duration = 5 # If duration is less than 0 then the trade is closed and the function should take less than 5 seconds to run
        else:
            duration = 5
        duration += 6
        
        self.logger.debug(f"Timeout set to: {duration} (6 extra seconds)")
        async def check(id):
            trade = await self.client.check_win(id)
            trade = json.loads(trade)
            win = trade["profit"]
            if win > 0:
                trade["result"] = "win"
            elif win == 0:
                trade["result"] = "draw"
            else:
                trade["result"] = "loss"
            return trade
        return await _timeout(check(id), duration)
        
        
    async def get_candles(self, asset: str, period: int, offset: int) -> list[dict]:  
        """
        Takes the asset you want to get the candles and return a list of raw candles in dictionary format
        Each candle contains:
            * time: using the iso format
            * open: open price
            * close: close price
            * high: highest price
            * low: lowest price
        """
        candles = await self.client.get_candles(asset, period, offset)
        return json.loads(candles)
    
    async def balance(self) -> float:
        "Returns the balance of the account"
        return json.loads(await self.client.balance())["balance"]
    
    async def opened_deals(self) -> list[dict]:
        "Returns a list of all the opened deals as dictionaries"
        return json.loads(await self.client.opened_deals())
    
    async def closed_deals(self) -> list[dict]:
        "Returns a list of all the closed deals as dictionaries"
        return json.loads(await self.client.closed_deals())
    
    async def clear_closed_deals(self) -> None:
        "Removes all the closed deals from memory, this function doesn't return anything"
        await self.client.clear_closed_deals()

    async def payout(self, asset: None | str | list[str] = None) -> dict | list[str] | int:
        "Returns a dict of asset : payout for each asset, if 'asset' is not None then it will return the payout of the asset or a list of the payouts for each asset it was passed"
        payout = json.loads(await self.client.payout())
        if isinstance(asset, str):
            return payout.get(asset)
        elif isinstance(asset, list):
            return [payout.get(ast) for ast in asset]
        return payout
    
    async def history(self, asset: str, period: int) -> list[dict]:
        "Returns a list of dictionaries containing the latest data available for the specified asset starting from 'period', the data is in the same format as the returned data of the 'get_candles' function."
        return json.loads(await self.client.history(asset, period))
    
    async def _subscribe_symbol_inner(self, asset: str) :
        return await self.client.subscribe_symbol(asset)
    
    async def _subscribe_symbol_chuncked_inner(self, asset: str, chunck_size: int):
        return await self.client.subscribe_symbol_chuncked(asset, chunck_size)
    
    async def _subscribe_symbol_timed_inner(self, asset: str, time: timedelta):
        return await self.client.subscribe_symbol_timed(asset, time)
    
    async def subscribe_symbol(self, asset: str) -> AsyncSubscription:
        """Returns an async iterator over the associated asset, it will return real time raw candles and will return new candles while the 'PocketOptionAsync' class is loaded if the class is droped then the iterator will fail"""
        return AsyncSubscription(await self._subscribe_symbol_inner(asset))
    
    async def subscribe_symbol_chuncked(self, asset: str, chunck_size: int) -> AsyncSubscription:
        """Returns an async iterator over the associated asset, it will return real time candles formed with the specified amount of raw candles and will return new candles while the 'PocketOptionAsync' class is loaded if the class is droped then the iterator will fail"""
        return AsyncSubscription(await self._subscribe_symbol_chuncked_inner(asset, chunck_size))
    
    async def subscribe_symbol_timed(self, asset: str, time: timedelta) -> AsyncSubscription:
        """
        Returns an async iterator over the associated asset, it will return real time candles formed with candles ranging from time `start_time` to `start_time` + `time` allowing users to get the latest candle of `time` duration and will return new candles while the 'PocketOptionAsync' class is loaded if the class is droped then the iterator will fail
        Please keep in mind the iterator won't return a new candle exactly each `time` duration, there could be a small delay and imperfect timestamps
        """
        return AsyncSubscription(await self._subscribe_symbol_timed_inner(asset, time))
    
    
async def _timeout(future, timeout: int):
    if sys.version_info[:3] >= (3,11):
        async with asyncio.timeout(timeout):
            return await future
    else:
        return await asyncio.wait_for(future, timeout)