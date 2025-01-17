from BinaryOptionsToolsV2.pocketoption import PocketOptionAsync
from BinaryOptionsToolsV2.tracing import start_logs
import random
import asyncio

async def main(ssid: str):
    start_logs(".", "debug", terminal=False)
    api = PocketOptionAsync(ssid)
    await asyncio.sleep(5)
    trades = 0
    while True:
        time = random.choice([5, 15, 30, 60, 300])
        amount = max(random.random() * 10, 1)
        print(f"Placing trade with timeframe {time} and amount {amount}")
        (id, _) = await api.buy("EURUSD_otc", amount, time, False)
        try: 
            res = await api.check_win(id)
            print(f"Result for trade: {res["result"]}",)
            trades += 1
        except Exception as e:
            print("---Second chance---")
            try: 
                (_, res) = await api.check_win(id)
                print(f"Result for trade: {res["result"]}",)
                trades += 1

            except Exception as e:
                print(f"Succesfully executed {trades} trades")
                raise e

if __name__ == "__main__":
    ssid = input("Please enter your ssid: ")
    asyncio.run(main(ssid))