from BinaryOptionsToolsV2.pocketoption import PocketOptionAsync
from BinaryOptionsToolsV2 import RawValidator

import asyncio

async def main(ssid: str):
    api = PocketOptionAsync(ssid)
    await asyncio.sleep(5)
    validator = RawValidator.starts_with(r'451-["signals/load"')
    res = await api.create_raw_order(r'42["signals/subscribe"]', validator)
    print(f"Recieved: {res}")
    
if __name__ == '__main__':
    ssid = input("Paste ssid: ")
    asyncio.run(main(ssid))