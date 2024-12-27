use chrono::{DateTime, Days, Utc};
use serde::Deserialize;
use reqwest::Client;
use std::env;
use std::str;

const BPM_ALERT_THRESHOLD: u8 = 80; // if bpm is above this, alert
const MINUTES_THRESHOLD: u64 = 60; // if it's been this many minutes since the last reading, alert

struct HeartData {
    bpm: u8,
    minutes_ago: u64,
}

#[derive(Deserialize, Debug)]
struct OuraHeartbeat {
    bpm: u8,
    #[allow(dead_code)]
    source: String,
    timestamp: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct OuraResponse {
    data: Vec<OuraHeartbeat>,
    #[allow(dead_code)]
    next_token: Option<String>,
}

// return bpm, and minutes since the last reading
async fn get_bpm_and_minutes_ago(now: DateTime<Utc>) -> HeartData {
    let token = env::var("OURA_ACCESS_TOKEN")
        .expect("Failed to get OURA_ACCESS_TOKEN environment variable");

    let yesterday = now.checked_sub_days(Days::new(1));

    let response = Client::new()
        .get(format!("https://api.ouraring.com/v2/usercollection/heartrate?start_datetime={}", yesterday.unwrap().format("%Y-%m-%dT%H:%M:%S")))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json::<OuraResponse>()
        .await
        .unwrap();

    let last_heartbeat = response.data.last();

    let chicago_tz: chrono_tz::Tz = chrono_tz::America::Chicago;
    let now_chicago = Utc::now().with_timezone(&chicago_tz);

    // Calculate the difference in minutes
    let duration = now_chicago.signed_duration_since(last_heartbeat.unwrap().timestamp);
    let minutes_ago = duration.num_minutes();

    HeartData {
        bpm: last_heartbeat.unwrap().bpm,
        minutes_ago: minutes_ago.try_into().unwrap(),
    }
}

fn alert_wrap(content: &str) -> String {
    format!(">>>>> {} <<<<<", content)
}

fn get_display(heart_data: HeartData) -> String {
    format!(
        "{} | {}",
        match heart_data.bpm >= BPM_ALERT_THRESHOLD {
            true => alert_wrap(&heart_data.bpm.to_string()),
            false => heart_data.bpm.to_string(),
        },
        match heart_data.minutes_ago >= MINUTES_THRESHOLD {
            true => alert_wrap(format!("{}m", heart_data.minutes_ago).as_str()), // data too stale
            false => format!("{}m", heart_data.minutes_ago),
        },
    )
}

#[tokio::main]
async fn main() {
    println!(
        "{}",
        get_display(get_bpm_and_minutes_ago(chrono::Utc::now()).await)
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_alert_wrap() {
        assert_eq!(alert_wrap("testing"), ">>>>> testing <<<<<");
    }

    #[test]
    fn test_get_display() {
        assert_eq!(
            get_display(HeartData {
                bpm: 70,
                minutes_ago: 10
            }),
            "70 | 10m"
        );
        assert_eq!(
            get_display(HeartData {
                bpm: 90,
                minutes_ago: 10
            }),
            ">>>>> 90 <<<<< | 10m"
        );
        assert_eq!(
            get_display(HeartData {
                bpm: 70,
                minutes_ago: 70
            }),
            "70 | >>>>> 70m <<<<<"
        );
        assert_eq!(
            get_display(HeartData {
                bpm: 90,
                minutes_ago: 70
            }),
            ">>>>> 90 <<<<< | >>>>> 70m <<<<<"
        );
    }
}
