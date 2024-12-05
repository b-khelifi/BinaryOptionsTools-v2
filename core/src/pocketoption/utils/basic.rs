use chrono::{Duration, Utc};
use rand::{thread_rng, Rng};

use crate::pocketoption::error::{PocketOptionError, PocketResult};


pub fn get_index() -> PocketResult<i128> {
    // rand = str(random.randint(10, 99))
    // cu = int(time.time())
    // t = str(cu + (2 * 60 * 60))
    // index = int(t + rand)
    let mut rng = thread_rng();

    let rand = rng.gen_range(10..99);
    let time = (Utc::now() + Duration::hours(2)).timestamp();
    format!("{}{}",time, rand ).parse::<i128>().map_err(|e| PocketOptionError::GeneralParsingError(e.to_string()))
}

