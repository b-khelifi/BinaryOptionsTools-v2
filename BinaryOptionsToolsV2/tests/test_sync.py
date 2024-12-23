from BinaryOptionsToolsV2.syncronous import PocketOption

def main(ssid, demo):
    api = PocketOption(ssid, demo)
    trade = api.buy("EURUSD", 1, 60, True)
    print(f"Trade: {trade}")

if __name__ == "__main__":
    ssid = input("Write your ssid: ")
    demo = True
    main(ssid, demo)