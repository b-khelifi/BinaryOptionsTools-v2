
import pandas as pd
import numpy as np
from sklearn.ensemble import RandomForestClassifier
from sklearn.model_selection import train_test_split
from sklearn.metrics import accuracy_score
import yfinance as yf
import asyncio
from pocketoptionapi.ws.client import WebsocketClient
from pocketoptionapi.ws.channels.buyv3 import Buyv3
from pocketoptionapi.ws.channels.get_balances import Get_Balances
from pocketoptionapi.ws.objects.timesync import TimeSync

# Step 1: Fetch Historical Data
def fetch_data(ticker, period='1y', interval='1h'):
    data = yf.download(ticker, period=period, interval=interval)
    data['Returns'] = data['Close'].pct_change()
    data.dropna(inplace=True)
    return data

# Step 2: Feature Engineering
def add_indicators(data):
    data['SMA_10'] = data['Close'].rolling(window=10).mean()
    data['SMA_50'] = data['Close'].rolling(window=50).mean()
    data['RSI'] = compute_rsi(data['Close'])
    data['Target'] = (data['Close'].shift(-1) > data['Close']).astype(int)
    data.dropna(inplace=True)
    return data

def compute_rsi(series, period=14):
    delta = series.diff()
    gain = (delta.where(delta > 0, 0)).rolling(window=period).mean()
    loss = (-delta.where(delta < 0, 0)).rolling(window=period).mean()
    rs = gain / loss
    rsi = 100 - (100 / (1 + rs))
    return rsi

# Step 3: Train the Model
def train_model(data):
    features = ['SMA_10', 'SMA_50', 'RSI']
    X = data[features]
    y = data['Target']

    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

    model = RandomForestClassifier(n_estimators=100, random_state=42)
    model.fit(X_train, y_train)

    predictions = model.predict(X_test)
    accuracy = accuracy_score(y_test, predictions)
    print(f"Model Accuracy: {accuracy:.2f}")

    return model

# Step 4: Simulate Trading
def simulate_trading(data, model):
    features = ['SMA_10', 'SMA_50', 'RSI']
    data['Signal'] = model.predict(data[features])

    data['Strategy'] = data['Signal'] * data['Returns']
    data['Cumulative_Strategy'] = (1 + data['Strategy']).cumprod()
    data['Cumulative_Market'] = (1 + data['Returns']).cumprod()

    print(f"Final Strategy Return: {data['Cumulative_Strategy'].iloc[-1]:.2f}")
    print(f"Final Market Return: {data['Cumulative_Market'].iloc[-1]:.2f}")

    return data

# Step 5: Pocket Option API Integration
class PocketOptionTrader:
    def __init__(self, api):
        self.api = api

    def connect(self):
        success, reason = self.api.connect()
        if success:
            print("Connected to Pocket Option API.")
        else:
            print(f"Failed to connect: {reason}")

    def place_order(self, ticker, action, amount):
        try:
            if action == "buy":
                result = self.api.buyv3.buy(amount, ticker, 1, 'turbo')
                print(f"Buy order placed: {result}")
            elif action == "sell":
                # Pocket Option API primarily deals with buying. Adjust for your needs.
                print("Sell order simulation completed.")
        except Exception as e:
            print(f"Failed to place order: {e}")

# Main Execution
def main():
    ticker = 'AAPL'  # Example: Apple Inc.
    data = fetch_data(ticker)
    data = add_indicators(data)

    model = train_model(data)
    results = simulate_trading(data, model)

    # Connect to Pocket Option
    api = WebsocketClient()
    trader = PocketOptionTrader(api)
    trader.connect()

    # Trading Simulation
    for i in range(len(results)):
        if results['Signal'].iloc[i] == 1:  # Buy Signal
            trader.place_order(ticker, "buy", 10)  # Replace 10 with desired amount
        elif results['Signal'].iloc[i] == 0:  # Sell Signal
            trader.place_order(ticker, "sell", 10)

    # Plot Results
    import matplotlib.pyplot as plt
    plt.figure(figsize=(12, 6))
    plt.plot(results['Cumulative_Strategy'], label='Strategy')
    plt.plot(results['Cumulative_Market'], label='Market')
    plt.legend()
    plt.title('Trading Strategy vs. Market Performance')
    plt.show()

if __name__ == "__main__":
    main()
