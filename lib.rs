use std::collections::HashMap;
use std::io::{BufReader,BufRead};
use std::fs::File;
use std::cmp;

extern crate regex;

pub type Dataset = HashMap<i32, Datapoint>;

pub struct Datapoint {
    pub value: u32,
    pub total: u32,
}

pub struct WDLData {
    pub wins: Dataset,
    pub draws: Dataset,
    pub losses: Dataset,
}

impl Datapoint {
    pub fn percentage_value(&self) -> f64 {
        (self.value as f64) / (self.total as f64)
    }
}

#[derive(Debug)]
pub enum GameResult { Win, Draw, Loss, Unknown }

pub fn calculate_from_files<'a, I>(files: I) -> String
where
    I: IntoIterator<Item = &'a str>
{
    String::from("ok")
}

fn calculate(file_path: &str) -> WDLData {
    let mut wins = Dataset::new();
    let mut draws = Dataset::new();
    let mut losses = Dataset::new();

    let tc_regex = regex::Regex::new("^\\[TimeControl \"(\\d+\\+\\d+)\"\\]$").unwrap();
    let whiteelo_regex = regex::Regex::new("^\\[WhiteElo \"(\\d+)\"\\]$").unwrap();
    let blackelo_regex = regex::Regex::new("^\\[BlackElo \"(\\d+)\"\\]$").unwrap();
    let result_regex = regex::Regex::new("^\\[Result \"[^\"]\"\\]$").unwrap();

    let file = BufReader::new(File::open(file_path).expect("One of the given files doesn't exist."));

    let mut skip = false;
    let mut rating1 = 0;
    let mut rating2 = 0;
    let mut result = GameResult::Unknown;

    for l in file.lines() {
        let line = l.unwrap();
        let line = line.trim();

        if tc_regex.is_match(line) {
            let tc = tc_regex.captures(line).unwrap().get(1).unwrap().as_str();
            let qualified_tc = timecontrol_qualifies(tc);
            skip = !qualified_tc;
        }
        else if whiteelo_regex.is_match(line) {
            rating1 = round_rating(whiteelo_regex.captures(line).unwrap().get(1).unwrap().as_str().parse().unwrap());
        }
        else if blackelo_regex.is_match(line) {
            rating2 = round_rating(blackelo_regex.captures(line).unwrap().get(1).unwrap().as_str().parse().unwrap());
        }
        else if result_regex.is_match(line) {
            let result_str = result_regex.captures(line).unwrap().get(1).unwrap().as_str();
            match result_str {
                "1-0" => result = GameResult::Win,
                "1/2-1/2" => result = GameResult::Draw,
                "0-1" => result = GameResult::Loss,
                _ => panic!("Unexpected result value {:?}", result),
            }
        }

        if !line.starts_with("[") && !skip {
            let min_rating = cmp::min(rating1, rating2);
            let max_rating = cmp::max(rating1, rating2);
            let rating_diff = max_rating - min_rating;

            if !wins.contains_key(&rating_diff) {
                wins.insert(rating_diff, Datapoint {value: 0, total: 0 });
            }
            wins.get_mut(&rating_diff).unwrap().total += 1;
            if !draws.contains_key(&rating_diff) {
                draws.insert(rating_diff, Datapoint {value: 0, total: 0 });
            }
            draws.get_mut(&rating_diff).unwrap().total += 1;
            if !losses.contains_key(&rating_diff) {
                losses.insert(rating_diff, Datapoint {value: 0, total: 0 });
            }
            losses.get_mut(&rating_diff).unwrap().total += 1;

            let mut relevant_set = match result {
                GameResult::Win => &mut wins,
                GameResult::Draw => &mut draws,
                GameResult::Loss => &mut losses,
                _ => panic!("result = Unknown")
            };
            relevant_set.get_mut(&rating_diff).unwrap().value += 1;
            skip = true;
        }
    }

    WDLData { wins, draws, losses }
}

fn timecontrol_qualifies(timecontrol: &str) -> bool {
    let mut parts = timecontrol.split("+");

    let initial: i32 = parts.next().unwrap().parse().unwrap();
    let increment: i32 = parts.next().unwrap().parse().unwrap();
    let estimate = initial + 40 * increment;

    estimate >= 480
}

fn round_rating(rating: i32) -> i32 {
    let rating = rating as f32;
    ((rating * 4.0 / 100.0).round() * 100.0 / 4.0) as i32
}

#[cfg(test)]
mod tests {
    use super::round_rating;
    #[test]
    pub fn test_round_rating() {
        assert_eq!(round_rating(1712), 1700);
        assert_eq!(round_rating(1713), 1725);
        assert_eq!(round_rating(848), 850);
        assert_eq!(round_rating(2093), 2100);
    }
}