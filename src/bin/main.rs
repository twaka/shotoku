use shotoku::*;

use chrono::Utc;
use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the server
    url: String,

    /// Path to the payload file
    payload_file: String,

    /// Number of virtual users
    #[arg(short = 'u', long, default_value_t = 1)]
    vus: usize,

    /// Duration of the test
    #[arg(short, long, default_value_t = 30)]
    duration: usize,

    /// Spawn rate of virtual users
    #[arg(short, long, default_value_t = 1)]
    spawn_rate: usize,
}

fn seconds_to_hms(seconds: usize) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let reader = BufReader::new(File::open(args.payload_file)?);

    let mut payloads = vec![];
    for line in reader.lines() {
        let line = line?;
        payloads.push(serde_json::from_str(&line)?);
    }

    let config = model::Config {
        vus: args.vus,
        duration: args.duration,
        spawn_rate: args.spawn_rate,
        url: args.url,
        // text_width: max(Term::stdout().size().1 as usize - 30, 0),
        text_width: 50,
    };

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner} {elapsed_precise}/{msg}").unwrap();
    let total_duration = config.duration + config.spawn_rate * config.vus;
    let pb = m.add(ProgressBar::new((total_duration * 10) as u64));
    pb.set_style(sty);
    pb.set_message(seconds_to_hms(total_duration));

    let (tx, mut rx) = mpsc::channel(100);

    let mut set = JoinSet::new();
    for rank in 0..config.vus {
        set.spawn(worker::worker(
            rank,
            payloads.clone(),
            m.clone(),
            config.clone(),
            tx.clone(),
        ));
    }

    drop(tx);

    let h = tokio::spawn(async move {
        let mut data = vec![];
        let mut heap = BinaryHeap::new();
        let window = 1;
        while let Some(worker_state) = rx.recv().await {
            heap.push(Reverse(worker_state.end));
            data.push(worker_state);
            let ts = Utc::now();
            while heap.peek().unwrap().0 < ts - Duration::from_secs(window) {
                heap.pop();
            }
            let toks = heap.len() as f64 / window as f64;
            pb.set_message(format!(
                "{} {:.2} aggregated tokens/s",
                seconds_to_hms(total_duration),
                toks
            ));
            pb.tick();
        }
        pb.finish();

        data
    });

    while let Some(_res) = set.join_next().await {}

    let data = h.await?;

    let ttfts = data
        .iter()
        .filter(|d| d.is_first)
        .map(|d| (d.end - d.begin).num_milliseconds())
        .collect::<Vec<_>>();
    println!(
        "average ttft: {:.2}ms",
        ttfts.iter().sum::<i64>() as f64 / ttfts.len() as f64
    );
    println!(
        "total token/s: {:.2}",
        data.len() as f64 / total_duration as f64
    );

    Ok(())
}
