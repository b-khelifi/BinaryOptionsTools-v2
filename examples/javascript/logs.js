const { PocketOption, startLogs } = require('@rick-29/binary-options-tools');

async function main(ssid) {
    // Start logs
    startLogs({
        path: ".",
        level: "DEBUG",
        terminal: true  // If false then the logs will only be written to the log files
    });
    
    // Initialize the API client
    const api = new PocketOption(ssid);
    
    // Place buy and sell orders
    const [buyId, buyData] = await api.buy({
        asset: "EURUSD_otc",
        amount: 1.0,
        time: 300,
        checkWin: false
    });
    
    const [sellId, sellData] = await api.sell({
        asset: "EURUSD_otc",
        amount: 1.0,
        time: 300,
        checkWin: false
    });
    
    console.log(buyId, sellId);
    
    // Check wins (same as setting checkWin to true in the buy/sell calls)
    const buyResult = await api.checkWin(buyId);
    const sellResult = await api.checkWin(sellId);
    
    console.log("Buy trade result:", buyResult.result);
    console.log("Buy trade data:", buyResult);
    console.log("Sell trade result:", sellResult.result);
    console.log("Sell trade data:", sellResult);
}

// Check if ssid is provided as command line argument
const ssid = process.argv[2];
if (!ssid) {
    console.log('Please provide your ssid as a command line argument');
    process.exit(1);
}

main(ssid).catch(console.error);