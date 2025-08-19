use crate::Maps;
use chrono::{FixedOffset, Utc};
use rand::Rng;
use std::collections::HashMap;

fn apply_function(function: &str, value: String, target: Option<&String>) -> String {
    let empty = "".to_string();
    match function {
        "send_date" => Utc::now().format("%-d/%m/%Y").to_string(),
        "send_hour" => send_hour(target),
        "random_num" => rand::rng().random_range(0..1000).to_string(),
        "downcase" => value.to_lowercase(),
        "upcase" => value.to_uppercase(),
        "first_word" => value.split(' ').next().unwrap_or("").to_string(),
        "first_down" => value.split(' ').next().unwrap_or("").to_lowercase(),
        "fixed" => target.unwrap_or(&empty).to_owned(),
        "dynamic" => value,
        _ => value,
    }
}

fn send_hour(target: Option<&String>) -> String {
    let tz_str: &str = target.map(|s| s.as_str()).unwrap_or("0:00");
    let tz = parse_timezone(tz_str);
    Utc::now().with_timezone(&tz).format("%H:%M").to_string()
}

fn parse_timezone(timezone: &str) -> FixedOffset {
    let neutral_time = timezone.replace('-', "");
    let parts: Vec<&str> = neutral_time.split(':').collect();
    let hours: i32 = parts[0].parse().unwrap_or(0);
    let minutes: i32 = parts[1].parse().unwrap_or(0);
    let seconds = hours * 3600 + minutes * 60;

    if timezone.contains('-') {
        FixedOffset::west_opt(seconds).unwrap()
    } else {
        FixedOffset::east_opt(seconds).unwrap()
    }
}

fn get_function_index(map: &HashMap<String, String>, headers: &[String]) -> Option<usize> {
    map.get("target").and_then(|target| headers
             .iter()
             .enumerate()
             .find(|(_, elem)| **elem == *target)
             .map(|(index, _)| index))
}

pub fn fill_row(new_row: &mut Vec<String>, functions: &Maps, row: &[String], headers: &[String]) {
    functions.keys().for_each(|key| {
        let map = functions.get(key).unwrap();
        let fun = map.get("fn").unwrap();
        let target = map.get("target");

        let value = get_function_index(map, headers).
            map_or_else(|| "".to_string(), |index| row[index].clone());

        new_row.push(apply_function(fun, value, target))
    });
}
