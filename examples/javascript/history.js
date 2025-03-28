const { PocketOption } = require('@rick-29/binary-options-tools');

async function main(ssid) {
    // Initialize the API client
    const api = new PocketOption(ssid);
    
    // Wait for connection to establish
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Get candles history
    const candles = await api.history("EURUSD_otc", 3600);
    console.log("Raw Candles:", candles);
    
    // If you want to use something similar to pandas in JavaScript,
    // you could use libraries like 'dataframe-js' or process the raw data
    const formattedCandles = candles.map(candle => ({
        time: new Date(candle.time).toISOString(),
        open: candle.open,
        high: candle.high,
        low: candle.low,
        close: candle.close,
        volume: candle.volume
    }));
    
    console.log("Formatted Candles:", formattedCandles);
}

// Check if ssid is provided as command line argument
const ssid = process.argv[2];
if (!ssid) {
    console.log('Please provide your ssid as a command line argument');
    process.exit(1);
}

main(ssid).catch(console.error);