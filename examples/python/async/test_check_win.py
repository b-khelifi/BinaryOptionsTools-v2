from BinaryOptionsToolsV2.pocketoption import PocketOptionAsync
from BinaryOptionsToolsV2.tracing import start_logs
import random
import asyncio

async def main(ssid: str):
    start_logs(".", "info", terminal=False)
    api = PocketOptionAsync(ssid)
    await asyncio.sleep(5)
    while True:
        time = random.choice([5, 15, 30, 60, 300])
        amount = max(random.random() * 10, 1)
        print(f"Placing trade with timeframe {time} and amount {amount}")
        (_, res) = await api.buy("EURUSD_otc", amount, time, True)
        print(f"Result for trade: {res["result"]}",)

if __name__ == "__main__":
    ssid = input("Write your SSID: ")
    asyncio.run(main(ssid))