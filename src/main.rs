use chrono::{DateTime, TimeDelta, Utc};
use std::env;
use std::process::{Command, Stdio};
use std::str;

const BPM_ALERT_THRESHOLD: u8 = 80; // if bpm is above this, alert
const MINUTES_THRESHOLD: u8 = 60; // if it's been this many minutes since the last reading, alert

struct HeartData {
    bpm: u8,
    minutes_ago: u8,
}

#[allow(clippy::identity_op)]
fn get_api_url(now: DateTime<Utc>) -> String {
    const START_TIME_DELTA: i64 = 2 * 60 * 60; // start n hours before now
    const END_TIME_DELTA: i64 = 1 * 60; // end n minutes after current time

    format!(
        "https://api.ouraring.com/v2/usercollection/heartrate?start_datetime={}&end_datetime={}",
        now.checked_sub_signed(
            TimeDelta::new(START_TIME_DELTA, 0).expect("Failed to get start TimeDelta")
        )
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S"),
        now.checked_add_signed(
            TimeDelta::new(END_TIME_DELTA, 0).expect("Failed to get end TimeDelta")
        )
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S")
    )
}

fn get_bpm_and_minutes_ago(now: DateTime<Utc>) -> HeartData {
    // return bpm, and minutes since the last reading

    let token = env::var("OURA_ACCESS_TOKEN")
        .expect("Failed to get OURA_ACCESS_TOKEN environment variable");

    let command = Command::new("curl")
        .arg("--silent")
        .arg("-X")
        .arg("GET")
        .arg(get_api_url(now))
        .arg("-H")
        .arg(format!("Authorization: Bearer {}", token))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let jq_command = Command::new("jq")
        .arg("-r")
        .arg(".data | last | \"\\(.bpm) \\(.timestamp)\"")
        .stdin(Stdio::from(command.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let output = jq_command.wait_with_output().unwrap();
    let result = str::from_utf8(&output.stdout).unwrap().to_string();
    let (bpm_str, timestamp_str) = result.trim().split_once(' ').unwrap();

    HeartData {
        bpm: bpm_str.parse::<u8>().unwrap(),
        minutes_ago: now
            .signed_duration_since(DateTime::parse_from_rfc3339(timestamp_str).unwrap())
            .num_minutes()
            .try_into()
            .unwrap(),
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

fn main() {
    println!(
        "{}",
        get_display(get_bpm_and_minutes_ago(chrono::Utc::now()))
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_api_url() {
        let api = get_api_url(chrono::Utc::now());
        assert!(api.contains("api.ouraring.com"));
    }

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
