const { PocketOption } = require('@rick-29/binary-options-tools');

async function main(ssid) {
    // Initialize the API client
    const api = new PocketOption(ssid);
    
    // Wait for connection to establish
    await new Promise(resolve => setTimeout(resolve, 5000));
    
    try {
        // Place a trade to get a trade ID
        const [tradeId, _] = await api.buy({
            asset: "EURUSD_otc",
            amount: 1.0,
            time: 300,
            checkWin: false
        });
        
        console.log(`Placed trade with ID: ${tradeId}`);
        
        // Get the deal end time
        const endTime = await api.getDealEndTime(tradeId);
        
        if (endTime) {
            const date = new Date(endTime * 1000);
            console.log(`Trade expires at: ${date.toLocaleString()}`);
            
            // Calculate time remaining
            const now = Math.floor(Date.now() / 1000);
            const remaining = endTime - now;
            console.log(`Time remaining: ${remaining} seconds`);
        } else {
            console.log("Could not find end time for trade");
        }
        
    } catch (error) {
        console.log(`Error: ${error}`);
    }
}

// Check if ssid is provided as command line argument
const ssid = process.argv[2];
if (!ssid) {
    console.log('Please provide your ssid as a command line argument');
    process.exit(1);
}

main(ssid).catch(console.error);