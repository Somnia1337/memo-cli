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
const MAX_OUT_FACTOR: f64 = 2.0;
const BASIC_WEIGHT: f64 = 10.0;
const MINIMUM_WEIGHT: f64 = 1.0;
const DECAY_RATE: f64 = 0.96;

const VAULT_NAME: &str = "memo";

#[cfg(target_os = "windows")]
const VAULT_PATH: &str = "D:/Markdown Files/Memo";

#[cfg(target_os = "macos")]
const VAULT_PATH: &str = "/Users/somnialu/Markdown/Memo";

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct ReviewInfo {
    file_name: String,
    last_reviewed: Option<NaiveDate>,
    review_count: u32,
}

impl ReviewInfo {
    fn new(file_name: String) -> Self {
        Self {
            file_name,
            last_reviewed: None,
            review_count: 0,
        }
    }
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

    let dir = format!("{}/{}", VAULT_PATH, subdir);
    let rev = format!("{}/revs/revs-{}.json", VAULT_PATH, subdir);

    let today = if let Some(date_str) = date {
        match NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => Local::now().date_naive(),
        }
    } else {
        Local::now().date_naive()
    };
    let loaded = load(&rev);

    let md_files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().is_some_and(|ext| ext == "md"))
        .map(|e| e.path().to_path_buf())
        .collect();

    let mut weights = Vec::new();
    let max_out = (md_files.len() as f64 * MAX_OUT_FACTOR / FILES_PER_DAY as f64).ceil() as i64;

    let mut review_data = HashMap::new();
    md_files.iter().for_each(|p| {
        let file_name = get_file_stem_str(p);
        let ri = if let Some(r) = loaded.iter().find(|r| r.file_name == file_name) {
            r.clone()
        } else {
            ReviewInfo::new(file_name.clone())
        };
        review_data.insert(file_name, ri);
    });

    let mut md_files: Vec<(PathBuf, usize)> = md_files
        .into_iter()
        .map(|f| {
            let file_name = get_file_stem_str(&f);
            (f.clone(), weight(&file_name, &review_data, today, max_out))
        })
        .collect();
    md_files.sort_by_key(|p| p.1);

    for (file, weight) in &md_files {
        let file_name = get_file_stem_str(file);
        let entry = review_data.entry(file_name.clone()).or_default();
        print!(
            "{:>3} | {:>10} | {:>2} | ",
            weight,
            entry
                .last_reviewed
                .map_or_else(|| "N/A".to_string(), |date| date.to_string()),
            entry.review_count,
        );
        show_link(file);
        weights.push((weight, file.clone()));
    }

    println!();
    let mut rng = rng();

    if top {
        weights.sort_unstable_by_key(|w| w.0);

        for _ in 0..FILES_PER_DAY {
            let file = weights.pop().unwrap().1;
            let path_str = show_link(&file);
            if !dry {
                modify(&mut review_data, path_str, today);
            }
        }

        if !dry {
            save(&review_data, &rev);
        }

        return;
    }

    let mut pool = Vec::new();
    for (w, f) in weights {
        for _ in 0..*w {
            pool.push(f.clone());
        }
    }

    pool.shuffle(&mut rng);
    let mut selected = HashSet::new();

    for file in pool.choose_multiple(&mut rng, pool.len()) {
        if selected.insert(file) && selected.len() == FILES_PER_DAY {
            break;
        }
    }

    for file in selected {
        let path_str = show_link(file);
        if !dry {
            modify(&mut review_data, path_str, today);
        }
    }

    if !dry {
        save(&review_data, &rev);
    }
}

fn get_file_stem_str(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn weight(
    file_name: &str,
    review_data: &HashMap<String, ReviewInfo>,
    today: NaiveDate,
    max_out: i64,
) -> usize {
    let info = review_data.get(file_name);
    let last_review = info.and_then(|i| i.last_reviewed);
    let review_count = info.map_or(0, |i| i.review_count);

    let priority_score = if review_count == 0 {
        100.0
    } else {
        let days_since_last = last_review.map_or(max_out, |d| (today - d).num_days().max(0));
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

fn show_link(file: &Path) -> String {
    let file_name = get_file_stem_str(file);
    if !file_name.is_empty() {
        let encoded = urlencoding::encode(&file_name);
        let uri = format!("obsidian://open?vault={}&file={}", VAULT_NAME, encoded);
        println!("\x1b]8;;{0}\x1b\\{1}\x1b]8;;\x1b\\", uri, file_name);

        file_name
    } else {
        String::new()
    }
}

fn modify(review_data: &mut HashMap<String, ReviewInfo>, file_name: String, today: NaiveDate) {
    review_data
        .entry(file_name.clone())
        .and_modify(|e| {
            e.last_reviewed = Some(today);
            e.review_count += 1;
        })
        .or_insert(ReviewInfo {
            file_name,
            last_reviewed: Some(today),
            review_count: 1,
        });
}

fn load(rev: &str) -> Vec<ReviewInfo> {
    fs::read_to_string(rev)
        .ok()
        .and_then(|data| serde_json::from_str::<Vec<ReviewInfo>>(&data).ok())
        .unwrap_or_default()
}

fn save(data: &HashMap<String, ReviewInfo>, rev: &str) {
    let mut data: Vec<_> = data.values().collect();
    data.sort_by_key(|d| &d.file_name);

    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = fs::write(rev, json);
    }
}
