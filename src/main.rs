use chrono::{DateTime, TimeDelta, Utc};
use std::env;
use std::process::{Command, Stdio};
use std::str;

const BPM_ALERT_THRESHOLD: u8 = 75; // if bpm is above this, alert
const MINUTES_THRESHOLD: u8 = 60; // if it's been this many minutes since the last reading, alert

fn get_api_url(now: DateTime<Utc>) -> String {
    const START_TIME_DELTA: i64 = 6 * 60 * 60; // start n hours before now
    const END_TIME_DELTA: i64 = 1 * 60; // end n minutes after current time

    let start_datetime = format!(
        "{}",
        now.checked_sub_signed(
            TimeDelta::new(START_TIME_DELTA, 0).expect("Failed to get start TimeDelta")
        )
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S")
    );
    let end_datetime = format!(
        "{}",
        now.checked_add_signed(
            TimeDelta::new(END_TIME_DELTA, 0).expect("Failed to get end TimeDelta")
        )
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%S")
    );
    format!(
        "https://api.ouraring.com/v2/usercollection/heartrate?start_datetime={}&end_datetime={}",
        start_datetime, end_datetime
    )
}

fn get_bpm_and_minutes_ago(now: DateTime<Utc>) -> (u8, u8) { // return bpm, and minutes since the last reading
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
    let (bpm_str, timestamp_str) = result.trim().split_once(" ").unwrap();

    let minutes_ago = now
        .signed_duration_since(DateTime::parse_from_rfc3339(timestamp_str).unwrap())
        .num_minutes();

    (bpm_str.parse::<u8>().unwrap(), minutes_ago.try_into().unwrap())
}

fn get_display(bpm: u8, minutes_ago: u8) -> String {
    const LEFT_ALERT: &str = ">>>>>";
    const RIGHT_ALERT: &str = "<<<<<";

    let bpm_str = match bpm {
        BPM_ALERT_THRESHOLD.. => {
            // bpm is too high
            format!("{} {} {}", LEFT_ALERT, bpm.to_string(), RIGHT_ALERT)
        }
        _ => bpm.to_string(),
    };
    let minutes_str = match minutes_ago {
        MINUTES_THRESHOLD.. => {
            // it's been too long since the last reading
            format!("{} {}m {}", LEFT_ALERT, minutes_ago.to_string(), RIGHT_ALERT)
        }
        _ => format!("{}m", minutes_ago)
    };
    format!("{} | {}", bpm_str, minutes_str)
}

fn main() {
    let now = chrono::Utc::now();
    let (bpm, minutes_ago) = get_bpm_and_minutes_ago(now);
    println!("{}", get_display(bpm, minutes_ago));
}
