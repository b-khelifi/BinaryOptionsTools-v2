use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AssetType {
    Stock,
    Currency,
    Commodity,
    Cryptocurrency,
    Index
}

#[derive(Serialize)]
pub struct LoadHistoryPeriod {
    pub active: String,
    pub period: i32,
    pub time: i64,
    pub index: i128,
    pub offset: i32
}


#[derive(Debug, Deserialize)]
struct TestDescerialize {
    command: String, 
    payload: Value
}

#[derive(Debug, Deserialize)]
struct Inner {
    data: Value
}

#[derive(Debug, Deserialize)]
struct UserInit {
    id: i32,
    secret: String,
}

#[derive(Debug, Deserialize)]
struct Root {
    command: String,
    payload: UserInit,
}

#[derive(Debug, Deserialize)]
struct OptionData {
    id: i32,
    symbol: String,
    name: String,
    asset_type: AssetType,
    in1: i32,
    in2: i32,
    in3: i32,
    in4: i32,
    in5: i32,
    in6: i32,
    in7: i32,
    in8: i32,
    arr: Vec<String>,
    in9: i128,
    val: bool,
    times: Vec<Time>,
    in10: i32,
    in11: i32,
    in12: i128
}

#[derive(Debug, Deserialize)]
struct Time {
    time: i32,
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, fs::File, io::BufReader};

    use serde_json::{Deserializer, Value};
    
    #[test]
    fn try_serialize() -> Result<(), Box<dyn Error>> {
        let test_1 = r#"42["user_init",{"id":87742848,"secret":"e7ae30ebe1421fd742dc9a73ab5acb80"}]"#;
        let parse1: Value = serde_json::from_str(&test_1.replace("42", ""))?;
        dbg!(parse1);

        let test_2 = r#"42["chat_room_list_update",{"message":{"room_id":14906,"user_id":4,"username":"Social Trading","date":1732744806,"message":" Top-3 traders of last 4 hour:🥇Manish Bhati ..+$50000.00🥈Wiktor..."}}]	"#;        
        let parse2: Value = serde_json::from_str(&test_1.replace("42", ""))?;
        dbg!(parse2);

        Ok(())
    }

    #[test]
    fn test_descerializer() -> Result<(), Box<dyn Error>>{
        let data = [  
        r#"42["user_init",{"id":87742848,"secret":"e7ae30ebe1421fd742dc9a73ab5acb80"}]"#,
        r#"42["chat_room_list",{"list":[{"id":14906,"type":2,"is_cost":0,"message":"Hello LaRaia","date":1732741192,"new":43913,"notify":1,"pinned":0,"mention":false,"tab":"chats","title":"General chat (English)","avatar":"default/chat-icon-en.png?v=1"},{"id":15397,"type":4,"is_cost":0,"message":"FOMC meeting minutes. USA, 21:00 (GMT+2) See more","date":1732730425,"new":629,"notify":1,"pinned":0,"mention":false,"tab":"notifications","title":"Analytics","avatar":"default/chat-icon-analitics.png?v=1"},{"id":6945,"type":4,"is_cost":0,"message":" Get Ready for the New Year Pocket Celebration '25!\r\rSimply...","date":1732545087,"new":1,"notify":1,"pinned":0,"mention":false,"tab":"notifications","title":"News","avatar":"default/chat-icon-news.png?v=1"},{"id":133868564,"type":7,"is_cost":0,"message":" Welcome to the Support Chat! Here you can get a quick...","date":1730384274,"new":1,"notify":1,"pinned":1,"mention":false,"tab":"chats","title":"Support Service","avatar":"/uploads/users/9d/e3/d9/2_user.png","user_id":2,"online":true},{"id":133868547,"type":10,"is_cost":0,"message":"You have a new message from support. See more","date":1730384265,"new":1,"notify":1,"pinned":0,"mention":false,"tab":"notifications","title":"System Pocket","avatar":"/uploads/users/41/05/98/3_user.png","user_id":3},{"id":133868546,"type":12,"is_cost":0,"message":" A new achievement unlocked: Real trading account is...","date":1730384265,"new":2,"notify":1,"pinned":0,"mention":false,"tab":"notifications","title":"System Pocket","avatar":"/uploads/users/41/05/98/3_user.png","user_id":3},{"id":8173,"type":4,"is_cost":0,"message":" Happy Bitcoin Pizza Day 2024!\r\rEach year on May 22,...","date":1716376375,"new":0,"notify":1,"pinned":0,"mention":false,"tab":"notifications","title":"Promo","avatar":"default/chat-icon-promo.png?v=1"}]}]"#,
        r#"451-["updateStream",{"_placeholder":true,"num":0}]	"#
        ];
        for item in data.iter() {
            let stream = Deserializer::from_str(item).into_iter::<Value>();
            let stream_lenght = stream.count();
            dbg!(stream_lenght);
            // for value in stream {
            //     let val: Result<TestDescerialize, serde_json::Error> = serde_json::from_value(value?);
            //     dbg!("Value: {:?}", val);
            // }
        }
        Ok(())
    }

    #[test]
    fn test_descerializer_file() -> Result<(), Box<dyn Error>> {
        let file = File::open("tests/data.txt")?;
        let reader = BufReader::new(file);
        let test: Vec<OptionData> = serde_json::from_reader(reader)?;
        dbg!(test.first());
        Ok(())
    }
}