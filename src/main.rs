use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use rand::rng;
use rand::seq::{IndexedRandom, SliceRandom};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

const FILES_PER_DAY: usize = 3;
const BASIC_WEIGHT: f64 = 10.0;
const MINIMUM_WEIGHT: f64 = 1.0;
const DECAY_RATE: f64 = 0.96;

const VAULT_NAME: &str = "memo";
const VAULT_PATH: &str = r"D:\Markdown Files\Memo";

#[derive(Parser)]
#[command(name = "memo", about = "Scientific memorizing helper.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long)]
    dry: bool,

    #[arg(long)]
    top: bool,

    #[arg(long, value_name = "DATE")]
    date: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Dive into the "101" subdir.
    #[command(alias = "101")]
    Code101,

    /// Dive into the "301" subdir.
    #[command(alias = "301")]
    Code301,

    /// Dive into the "408" subdir.
    #[command(alias = "408")]
    Code408,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct ReviewInfo {
    last_reviewed: Option<NaiveDate>,
    review_count: u32,
}

fn main() {
    let Cli {
        command,
        dry,
        top,
        date,
    } = Cli::parse();

    let subdir = match command {
        Commands::Code101 => "101",
        Commands::Code301 => "301",
        Commands::Code408 => "408",
    };

    let dir = format!(r"{}\{}", VAULT_PATH, subdir);
    let rev = format!(r"{}\revs\revs-{}.json", VAULT_PATH, subdir);

    let today = if let Some(date_str) = date {
        match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => Local::now().date_naive(),
        }
    } else {
        Local::now().date_naive()
    };
    let mut review_data: HashMap<String, ReviewInfo> = load(&rev);

    let md_files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut pool = Vec::new();
    let mut weights = Vec::new();
    let total_days = (md_files.len() as f64 * 2.0 / FILES_PER_DAY as f64).ceil() as i64;

    let mut md_files: Vec<(PathBuf, usize)> = md_files
        .into_iter()
        .map(|f| (f.clone(), weight(&f, &review_data, today, total_days)))
        .collect();
    md_files.sort_by_key(|p| p.1);

    for (file, weight) in &md_files {
        let file_name = file.to_string_lossy().to_string();
        let entry = review_data.entry(file_name).or_default();
        println!(
            "{:>3} | {:>10} | {:>2} | {}",
            weight,
            entry
                .last_reviewed
                .map_or_else(|| "N/A".to_string(), |date| date.to_string()),
            entry.review_count,
            file.file_stem().unwrap().to_string_lossy(),
        );

        if top {
            weights.push((weight, file.clone()));
        } else {
            for _ in 0..*weight {
                pool.push(file.clone());
            }
        }
    }

    println!();
    let mut rng = rng();

    if top {
        weights.shuffle(&mut rng);
        weights.sort_unstable_by_key(|w| w.0);

        for _ in 0..FILES_PER_DAY {
            let file = weights.pop().unwrap().1;
            let path_str = show_link(&file, &review_data);
            modify(&mut review_data, path_str, today);
        }

        if !dry {
            save(&review_data, &rev);
        }

        return;
    }

    pool.shuffle(&mut rng);
    let mut unique_files = HashSet::new();
    let mut selected = Vec::new();

    for file in pool.choose_multiple(&mut rng, pool.len()) {
        if unique_files.insert(file) {
            selected.push(file.clone());
            if selected.len() == FILES_PER_DAY {
                break;
            }
        }
    }

    for file in &selected {
        let path_str = show_link(file, &review_data);
        modify(&mut review_data, path_str, today);
    }

    if !dry {
        save(&review_data, &rev);
    }
}

fn weight(
    file: &Path,
    review_data: &HashMap<String, ReviewInfo>,
    today: NaiveDate,
    total_days: i64,
) -> usize {
    let path_str = file.to_string_lossy().to_string();
    let info = review_data.get(&path_str);
    let last_review = info.and_then(|i| i.last_reviewed);
    let review_count = info.map_or(0, |i| i.review_count);

    let priority_score = if review_count == 0 {
        100.0
    } else {
        let days_since_last = last_review.map_or(total_days, |d| (today - d).num_days().max(0));
        let retention = DECAY_RATE.powi(days_since_last as i32);
        (1.0 - retention) * 100.0
    };

    let review_penalty = if review_count == 0 {
        0.0
    } else {
        (review_count as f64).ln() * 5.0
    };

    let adjusted_weight = (BASIC_WEIGHT + priority_score - review_penalty).max(MINIMUM_WEIGHT);

    adjusted_weight.round() as usize
}

fn show_link(file: &Path, review_data: &HashMap<String, ReviewInfo>) -> String {
    let path_str = file.to_string_lossy().to_string();
    let cur_count = review_data
        .get(&path_str)
        .map(|e| e.review_count)
        .unwrap_or_default()
        + 1;

    if let Some(file_name) = file.file_stem() {
        let file_name = file_name.to_string_lossy();
        let encoded = urlencoding::encode(&file_name);
        let uri = format!("obsidian://open?vault={}&file={}", VAULT_NAME, encoded);
        println!(
            "\x1b]8;;{0}\x1b\\{1} ({2})\x1b]8;;\x1b\\",
            uri, file_name, cur_count
        );
    }

    path_str
}

fn modify(review_data: &mut HashMap<String, ReviewInfo>, path_str: String, today: NaiveDate) {
    review_data
        .entry(path_str)
        .and_modify(|e| {
            e.review_count += 1;
            e.last_reviewed = Some(today);
        })
        .or_insert(ReviewInfo {
            last_reviewed: Some(today),
            review_count: 1,
        });
}

fn load(rev: &str) -> HashMap<String, ReviewInfo> {
    fs::read_to_string(rev)
        .ok()
        .and_then(|data| serde_json::from_str::<Vec<(String, ReviewInfo)>>(&data).ok())
        .unwrap_or_default()
        .into_iter()
        .collect()
}

fn save(data: &HashMap<String, ReviewInfo>, rev: &str) {
    let mut data: Vec<_> = data.iter().collect();
    data.sort_by_key(|d| d.0);

    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(rev, json);
    }
}
