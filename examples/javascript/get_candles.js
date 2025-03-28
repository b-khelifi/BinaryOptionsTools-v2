const { PocketOption } = require('@rick-29/binary-options-tools');

async function main(ssid) {
    // Initialize the API client
    const api = new PocketOption(ssid);
    
    // Wait for connection to establish
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    // Define time ranges and frames
    const times = Array.from({length: 10}, (_, i) => 3600 * (i + 1));
    const timeFrames = [1, 5, 15, 30, 60, 300];
    
    // Get candles for each combination
    for (const time of times) {
        for (const frame of timeFrames) {
            const candles = await api.getCandles("EURUSD_otc", 60, time);
            console.log(`Candles for time ${time} and frame ${frame}:`, candles);
        }
    }
}

// Check if ssid is provided as command line argument
const ssid = process.argv[2];
if (!ssid) {
    console.log('Please provide your ssid as a command line argument');
    process.exit(1);
}

main(ssid).catch(console.error);