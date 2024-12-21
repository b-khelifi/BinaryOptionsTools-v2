import asyncio
# import json

# import BinaryOptionsToolsV2
# from BinaryOptionsToolsV2 import connect


# print(BinaryOptionsToolsV2)
from asyncronous import PocketOptionAsync, async_connect

# async def main():
#     api = await connect(ssid, True)
#     (id, trade) = await api.buy("EURUSD", 1, 60)
    
#     print(f"Id: {id}")
#     print(f"Trade: {trade}")
#     result = await api.check_win(id)
#     result = json.loads(result)
#     print(f"Trade result: {result['profit']}")
#     print(f"Command: {result['command']}")
#     await asyncio.sleep(1)
#     balance = await api.balance()
#     print(f"Balance: {balance}")

async def main(ssid, demo):
    api = await async_connect(ssid, demo)
    # trade = await api.buy("EURUSD", 1.5, 60, True)
    # print(f"Trade: {trade}")
    await asyncio.sleep(10)
    candles = await api.get_candles("EURUSD_otc", 1, 60)
    print(f"Candles: {candles}")
    
if __name__ == "__main__":
    ssid = input("Write your ssid: ")
    demo = False
    asyncio.run(main(ssid, demo))
