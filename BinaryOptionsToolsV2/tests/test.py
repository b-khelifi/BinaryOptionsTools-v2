import asyncio
import pandas as pd # type: ignore
# import json

# import BinaryOptionsToolsV2
# from BinaryOptionsToolsV2 import connect


# print(BinaryOptionsToolsV2)
from BinaryOptionsToolsV2.asyncronous import async_connect

async def main(ssid):
    api = await async_connect(ssid)
    await asyncio.sleep(10)
    payout = await api.payout()
    candles = await api.history("EURUSD_otc", 7200)
    trade = await api.buy("EURUSD_otc", 1, 5)
    print(f"Payout: {payout}")
    print(f"Candles: {candles}")
    print(f"Trade: {trade}")
    df = pd.DataFrame.from_dict(candles)
    df.to_csv("candles_eurusd_otc.csv")    
if __name__ == "__main__":
    ssid = input("Write your ssid: ")
    demo = True
    asyncio.run(main(ssid))
