use std::collections::HashMap;
use std::io::{BufReader,BufRead};
use std::fs::File;
use std::cmp;

extern crate regex;

trait Aggregatable {
    fn aggregate(self, other: Self) -> Self;
}

pub type Dataset = HashMap<i32, Datapoint>;

#[derive(Clone)]
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

impl Aggregatable for Datapoint {
    fn aggregate(self, other: Datapoint) -> Datapoint {
        Datapoint {
            value: self.value + other.value,
            total: self.total + other.total,
        }
    }
}

impl Aggregatable for Dataset {
    fn aggregate(self, other: Dataset) -> Dataset {
        let mut new_set = self.clone();

        for key in other.keys() {
            if !new_set.contains_key(key) {
                new_set.insert(*key, other.get(key).unwrap().clone());
            }
            else {
                *new_set.get_mut(key).unwrap() = new_set.get(key).unwrap().clone().aggregate(other.get(key).unwrap().clone());
            }
        }

        new_set
    }
}

impl Aggregatable for WDLData {
    fn aggregate(self, other: WDLData) -> WDLData {
        WDLData {
            wins: self.wins.aggregate(other.wins),
            draws: self.draws.aggregate(other.draws),
            losses: self.losses.aggregate(other.losses),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum GameResult { Win, Draw, Loss, Unknown, Unfinished }

pub fn calculate_from_files<'a, I>(files: I)
where
    I: IntoIterator<Item = &'a str>
{
    let mut total_data = WDLData {
        wins: Dataset::new(),
        draws: Dataset::new(),
        losses: Dataset::new(),
    };
    for path in files {
        total_data = calculate(path).aggregate(total_data);
    }
    wdldata_presentation(&total_data);
}

fn calculate(file_path: &str) -> WDLData {
    let mut wins = Dataset::new();
    let mut draws = Dataset::new();
    let mut losses = Dataset::new();

    let tc_regex = regex::Regex::new("^\\[TimeControl \"(\\d+\\+\\d+)\"\\]$").unwrap();
    let whiteelo_regex = regex::Regex::new("^\\[WhiteElo \"(\\d+)\"\\]$").unwrap();
    let blackelo_regex = regex::Regex::new("^\\[BlackElo \"(\\d+)\"\\]$").unwrap();
    let result_regex = regex::Regex::new("^\\[Result \"([^\"]+)\"\\]$").unwrap();

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
                "*" => result = GameResult::Unfinished,
                _ => panic!("Unexpected result value {:?}", result_str),
            }
        }

        if !line.starts_with("[") && !skip && result != GameResult::Unfinished {
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

fn wdldata_presentation(data: &WDLData) {
    println!("Wins");
    println!("----------");
    for key in data.wins.keys() {
        let point = data.wins.get(key).unwrap();
        println!("+{}: {} ({}/{})", *key, point.percentage_value(), point.value, point.total);
    }
    println!("----------");
    println!("Draws");
    println!("----------");
    for key in data.draws.keys() {
        let point = data.draws.get(key).unwrap();
        println!("+{}: {} ({}/{})", *key, point.percentage_value(), point.value, point.total);
    }
    println!("----------");
    println!("Losses");
    println!("----------");
    for key in data.losses.keys() {
        let point = data.losses.get(key).unwrap();
        println!("+{}: {} ({}/{})", *key, point.percentage_value(), point.value, point.total);
    }
    println!("----------");
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