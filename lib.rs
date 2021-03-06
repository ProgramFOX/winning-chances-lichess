use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::cmp;

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
            } else {
                *new_set.get_mut(key).unwrap() = new_set
                    .get(key)
                    .unwrap()
                    .clone()
                    .aggregate(other.get(key).unwrap().clone());
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameResult {
    Win,
    Draw,
    Loss,
    Unknown,
    Unfinished,
}

impl std::ops::Not for GameResult {
    type Output = GameResult;

    fn not(self) -> GameResult {
        match self {
            GameResult::Win => GameResult::Loss,
            GameResult::Loss => GameResult::Win,
            other => other,
        }
    }
}

pub fn calculate_from_files<'a, I>(files: I)
where
    I: IntoIterator<Item = &'a str>,
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
    let mut processed: u32 = 0;

    let mut wins = Dataset::new();
    let mut draws = Dataset::new();
    let mut losses = Dataset::new();

    let file =
        BufReader::new(File::open(file_path).expect("One of the given files doesn't exist."));

    let mut skip = false;
    let mut rating1 = 0;
    let mut rating2 = 0;
    let mut result_white_pov = GameResult::Unknown;

    for l in file.lines() {
        let line = l.unwrap();

        if line.starts_with("[TimeControl") {
            let tc = line.split("\"").nth(1).unwrap();
            let qualified_tc = timecontrol_qualifies(tc);
            skip = !qualified_tc;

            processed += 1;
            if processed % 100000 == 0 {
                println!("{} processed in the current file", processed);
            }
        } else if line.starts_with("[WhiteElo") {
            rating1 = line.split("\"").nth(1).unwrap().parse().unwrap();
        } else if line.starts_with("[BlackElo") {
            rating2 = line.split("\"").nth(1).unwrap().parse().unwrap();
        } else if line.starts_with("[Result") {
            let result_str = line.split("\"").nth(1).unwrap();
            match result_str {
                "1-0" => result_white_pov = GameResult::Win,
                "1/2-1/2" => result_white_pov = GameResult::Draw,
                "0-1" => result_white_pov = GameResult::Loss,
                "*" => result_white_pov = GameResult::Unfinished,
                _ => panic!("Unexpected result value {:?}", result_str),
            }
        }

        if !line.starts_with("[") && !skip && result_white_pov != GameResult::Unfinished
            && rating1 != rating2
        {
            let min_rating = cmp::min(rating1, rating2);
            let max_rating = cmp::max(rating1, rating2);
            let rating_diff = round_rating(max_rating) - round_rating(min_rating);

            let result_lowest_pov = if min_rating == rating1 {
                // White is the lowest rated player.
                result_white_pov
            } else {
                // Black is the lowest rated player.
                !result_white_pov
            };

            if !wins.contains_key(&rating_diff) {
                wins.insert(rating_diff, Datapoint { value: 0, total: 0 });
            }
            wins.get_mut(&rating_diff).unwrap().total += 1;
            if !draws.contains_key(&rating_diff) {
                draws.insert(rating_diff, Datapoint { value: 0, total: 0 });
            }
            draws.get_mut(&rating_diff).unwrap().total += 1;
            if !losses.contains_key(&rating_diff) {
                losses.insert(rating_diff, Datapoint { value: 0, total: 0 });
            }
            losses.get_mut(&rating_diff).unwrap().total += 1;

            let mut relevant_set = match result_lowest_pov {
                GameResult::Win => &mut wins,
                GameResult::Draw => &mut draws,
                GameResult::Loss => &mut losses,
                _ => panic!("result = Unknown"),
            };
            relevant_set.get_mut(&rating_diff).unwrap().value += 1;
            skip = true;
        }
    }

    WDLData {
        wins,
        draws,
        losses,
    }
}

fn timecontrol_qualifies(timecontrol: &str) -> bool {
    if !timecontrol.contains("+") {
        return false;
    }

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
    let mut win_strings: Vec<(i32, String)> = vec![];
    for key in data.wins.keys() {
        let point = data.wins.get(key).unwrap();
        win_strings.push((
            *key,
            format!(
                "+{}: {} ({}/{})",
                *key,
                point.percentage_value(),
                point.value,
                point.total
            ),
        ));
    }
    win_strings.sort_by(|a, b| a.0.cmp(&b.0));
    for s in win_strings {
        println!("{}", s.1);
    }
    println!("----------");
    println!("Draws");
    println!("----------");
    let mut draw_strings: Vec<(i32, String)> = vec![];
    for key in data.draws.keys() {
        let point = data.draws.get(key).unwrap();
        draw_strings.push((
            *key,
            format!(
                "+{}: {} ({}/{})",
                *key,
                point.percentage_value(),
                point.value,
                point.total
            ),
        ));
    }
    draw_strings.sort_by(|a, b| a.0.cmp(&b.0));
    for s in draw_strings {
        println!("{}", s.1);
    }
    println!("----------");
    println!("Losses");
    println!("----------");
    let mut loss_strings: Vec<(i32, String)> = vec![];
    for key in data.losses.keys() {
        let point = data.losses.get(key).unwrap();
        loss_strings.push((
            *key,
            format!(
                "+{}: {} ({}/{})",
                *key,
                point.percentage_value(),
                point.value,
                point.total
            ),
        ));
    }
    loss_strings.sort_by(|a, b| a.0.cmp(&b.0));
    for s in loss_strings {
        println!("{}", s.1);
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
