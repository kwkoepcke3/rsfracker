use ordered_float::OrderedFloat;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct CliArgs {
    words: Vec<String>,
}

struct Score<'a> {
    guess: &'a String,
    buckets: HashMap<usize, Vec<String>>,
    entropy: f32,
    worst: usize,
    p_win: f32,
    exp_remaining: f32,
}

fn entropy_guess_tree(candidates: &Vec<String>) -> String {
    if candidates.len() == 1 {
        return candidates[0].clone();
    }

    let total = candidates.len() as f32;

    let scores: Vec<Score> = candidates
        .iter()
        .map(|guess| {
            let mut buckets: HashMap<usize, Vec<String>> = HashMap::new();
            for cand in candidates.iter() {
                let k = matches(guess, cand);
                buckets.entry(k).or_default().push(cand.to_owned());
            }

            let probs: Vec<f32> = buckets.values().map(|ws| ws.len() as f32 / total).collect();
            let entropy: f32 = -probs
                .iter()
                .filter(|&&p| p > 0.0)
                .map(|&p| p * p.log2())
                .sum::<f32>();
            let worst: usize = buckets.values().map(|ws| ws.len()).max().unwrap_or(0);
            let p_win: f32 =
                (buckets.get(&guess.len()).map(|ws| ws.len()).unwrap_or(0) as f32) / total;

            let exp_remaining: f32 = buckets
                .values()
                .map(|ws| {
                    let s = ws.len() as f32;
                    s * s
                })
                .sum::<f32>()
                / total;

            Score {
                guess,
                buckets,
                entropy,
                worst,
                p_win,
                exp_remaining,
            }
        })
        .collect();

    let best = scores
        .iter()
        .max_by(|a, b| {
            let key_a = (
                OrderedFloat(a.entropy),                          // ↑
                std::cmp::Reverse(a.worst),                       // ↓
                OrderedFloat(a.p_win),                            // ↑
                std::cmp::Reverse(OrderedFloat(a.exp_remaining)), // ↓
                std::cmp::Reverse(a.guess),                       // lexicographically smallest wins
            );
            let key_b = (
                OrderedFloat(b.entropy),
                std::cmp::Reverse(b.worst),
                OrderedFloat(b.p_win),
                std::cmp::Reverse(OrderedFloat(b.exp_remaining)),
                std::cmp::Reverse(b.guess),
            );
            key_a.cmp(&key_b)
        })
        .expect("No guesses found");

    let best_word = best.guess;
    let buckets = &best.buckets;
    let best_ent = best.entropy;

    println!("Best word to choose: {best_word} with entropy {best_ent}");
    print!("Num Correct (0..={}):", best_word.len());
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input: usize = input.trim().parse().expect("Could not parse input!");
    assert!(input <= best_word.len(), "Input too large!");

    let rest = buckets.get(&input).expect("Could not get bucket");

    return entropy_guess_tree(rest);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    let split_on = Regex::new(r"\W+")?;

    let mut candidates = if args.words.len() == 1 {
        let path = Path::new(&args.words[0]);

        let words: String = fs::read_to_string(path)?;

        split_on
            .split(&words)
            .map(|w| w.to_owned())
            .collect::<Vec<String>>()
    } else {
        args.words
    };

    candidates.dedup();
    candidates.sort();

    let result = entropy_guess_tree(&candidates);

    println!("Answer found! {result}");

    return Ok(());
}

fn matches(g: &str, w: &str) -> usize {
    g.chars().zip(w.chars()).filter(|(a, b)| a == b).count()
}
