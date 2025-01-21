pub mod contstants;
pub mod error;
pub mod general;
pub mod pocketoption;
pub mod utils;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use macros::{deserialize, serialize, timeout};
    use serde::{Deserialize, Serialize};
    use tokio::time::sleep;
    use tracing::debug;

    use crate::utils::tracing::start_tracing;
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Test {
        name: String
    }
    
    #[test]
    fn test_deserialize_macro() {
        let test = Test { name: "Test".to_string() };
        let test_str = serialize!(&test).unwrap();
        let test2 = deserialize!(Test, &test_str).unwrap();
        assert_eq!(test, test2)
    }

    struct Tester;

    #[tokio::test]
    async fn test_timeout_macro() -> anyhow::Result<()> {
        start_tracing(true).unwrap();
        
        #[timeout(1, tracing(level = "info", skip(tester)))]
        async fn this_is_a_test(tester: Tester) -> anyhow::Result<()> {
            debug!("Test");
            sleep(Duration::from_secs(0)).await;
            Ok(())
        }

        this_is_a_test(Tester,).await?
    }
}