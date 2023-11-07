use tokio::time::sleep;
use std::time::Duration;
use chrono::Local;

pub async fn get_looper() -> String {
    let mut timer : u8 = 20;
    loop {
        match reqwest::get("https://na-msk.ru/schedule-member/").await {
            Ok(value) => {
                println!("Connection established at {}\n", Local::now().format("[%Y-%m-%d][%H:%M:%S]"));
                return value.text().await.expect("Couldn't reformat the data to string code")
            }
            Err(e) => {
                println!("Error with connection! Retrying to connect in 60 secs.{}\n{}", Local::now().format("[%Y-%m-%d][%H:%M:%S]"), e);
                loop {
                    if timer == 0 {
                        println!("Time is out, trying to reconnect at {}", Local::now().format("[%Y-%m-%d][%H:%M:%S]"));
                        timer = 60;
                        break
                    }
                    else {
                        sleep(Duration::from_secs(1)).await;
                        timer -= 1
                    }
                }
            }
        }
    }
}