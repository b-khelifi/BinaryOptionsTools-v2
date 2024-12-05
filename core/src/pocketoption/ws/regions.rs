use crate::pocketoption::{error::PocketResult, utils::location::{calculate_distance, get_public_ip, get_user_location}};


pub struct Regions;

impl Regions {
    pub const DEMO: &str = "wss://demo-api-eu.po.market/socket.io/?EIO=4&transport=websocket";

    pub const EUROPE: &str = "wss://api-eu.po.market/socket.io/?EIO=4&transport=websocket";
    pub const SEYCHELLES: &str = "wss://api-sc.po.market/socket.io/?EIO=4&transport=websocket";
    pub const HONG_KONG: &str = "wss://api-hk.po.market/socket.io/?EIO=4&transport=websocket";
    pub const RUSSIA_SPB: &str = "wss://api-spb.po.market/socket.io/?EIO=4&transport=websocket";
    pub const FRANCE_2: &str = "wss://api-fr2.po.market/socket.io/?EIO=4&transport=websocket";
    pub const US_WEST_4: &str = "wss://api-us4.po.market/socket.io/?EIO=4&transport=websocket";
    pub const US_WEST_3: &str = "wss://api-us3.po.market/socket.io/?EIO=4&transport=websocket";
    pub const US_WEST_2: &str = "wss://api-us2.po.market/socket.io/?EIO=4&transport=websocket";
     
    pub const US_NORTH: &str = "wss://api-us-north.po.market/socket.io/?EIO=4&transport=websocket";
    pub const RUSSIA_MOSCOW: &str = "wss://api-msk.po.market/socket.io/?EIO=4&transport=websocket";
    pub const LATIN_AMERICA: &str = "wss://api-l.po.market/socket.io/?EIO=4&transport=websocket";
    pub const INDIA: &str = "wss://api-in.po.market/socket.io/?EIO=4&transport=websocket";
    pub const FRANCE: &str = "wss://api-fr.po.market/socket.io/?EIO=4&transport=websocket";
    pub const FINLAND: &str = "wss://api-fin.po.market/socket.io/?EIO=4&transport=websocket";
    pub const CHINA: &str = "wss://api-c.po.market/socket.io/?EIO=4&transport=websocket";
    pub const ASIA: &str = "wss://api-asia.po.market/socket.io/?EIO=4&transport=websocket";


    async fn get_closest_server(&self, ip_address: &str) -> PocketResult<Vec<(&str, f64)>> {
        let user_location = get_user_location(ip_address).await?;
        
        let server_locations = [
            (Self::EUROPE, 50.0, 10.0),
            (Self::SEYCHELLES, -4.0, 55.0),
            (Self::HONG_KONG, 22.0, 114.0),
            (Self::RUSSIA_SPB, 60.0, 30.0),
            (Self::FRANCE_2, 46.0, 2.0),
            (Self::US_WEST_4, 37.0, -122.0),
            (Self::US_WEST_3, 34.0, -118.0),
            (Self::US_WEST_2, 39.0, -77.0),
            (Self::US_NORTH, 42.0, -71.0),
            (Self::RUSSIA_MOSCOW, 55.0, 37.0),
            (Self::LATIN_AMERICA, 0.0, -45.0),
            (Self::INDIA, 20.0, 77.0),
            (Self::FRANCE, 46.0, 2.0),
            (Self::FINLAND, 62.0, 27.0),
            (Self::CHINA, 35.0, 105.0),
            (Self::ASIA, 10.0, 100.0),
        ];
    
        let mut distances: Vec<(&'static str, f64)> = server_locations
            .iter()
            .map(|(name, lat, lon)| {
                let distance = calculate_distance(user_location.0, user_location.1, *lat, *lon);
                (*name, distance)
            })
            .collect();
    
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    
        Ok(distances)
    }

    pub async fn get_servers(&self) -> PocketResult<Vec<&str>> {
        let ip = get_public_ip().await?;
        let servers = self.get_closest_server(&ip).await?;
        Ok(servers.iter().map(|(server, _)| *server).collect())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_closest_server() -> anyhow::Result<()> {
        let ip = get_public_ip().await?;
        let server = Regions.get_closest_server(&ip).await?;
        dbg!(server);
        Ok(())
    }
}