use crate::model::{Config, WorkerState};
use chrono::Utc;
use console::measure_text_width;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Deserialize;
use serde_json::Value;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::sync::mpsc;

pub async fn worker(
    rank: usize,
    payloads: Vec<Value>,
    m: MultiProgress,
    config: Config,
    tx: mpsc::Sender<WorkerState>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // initial delay
    tokio::time::sleep(Duration::from_secs((config.spawn_rate * rank) as u64)).await;
    let client = reqwest::Client::new();
    let begin = Utc::now();

    let sty = ProgressStyle::with_template("{spinner} {msg}").unwrap();
    let pb = m.add(ProgressBar::new_spinner());
    pb.set_style(sty);

    let mut last_success = Utc::now();

    let mut ite = 0;
    while Utc::now() - begin < chrono::Duration::seconds(config.duration as i64) {
        let payload = &payloads[(rank + ite * config.vus) % payloads.len()];
        pb.set_message("new request sent...");

        let response = client.post(&config.url).json(&payload).send().await;

        let response = match response {
            Ok(res) => res,
            Err(e) => {
                pb.set_message(format!("error: {}", e));
                pb.tick();
                continue;
            }
        };

        if !response.status().is_success() {
            pb.set_message(format!("error: {}", response.status()));
            pb.tick();
            continue;
        }

        let mut stream = response.bytes_stream().eventsource();

        let mut que = VecDeque::new();
        let mut ttft = 0;
        let mut is_first = true;
        while let Some(thing) = stream.next().await {
            #[derive(Deserialize, Debug)]
            struct Data {
                content: String,
            }
            let Data { content, .. } = serde_json::from_str(&thing?.data)?;
            if content == "" {
                continue;
            }

            let ts = Utc::now();

            content.chars().for_each(|x| {
                que.push_back(x.escape_debug().to_string());
            });

            tx.send(WorkerState {
                rank,
                ite,
                is_first,
                begin: last_success,
                end: ts,
                content,
            })
            .await?;

            let mut toks = 0.0;
            if is_first {
                is_first = false;
                ttft = (ts - last_success).num_milliseconds();
            } else {
                toks = 1000.0 / ((ts - last_success).num_milliseconds() as f64);
            }

            let mut length = 0;
            que.iter().for_each(|x| {
                length += measure_text_width(x);
            });

            while length > config.text_width {
                length -= measure_text_width(&que[0]);
                que.pop_front();
            }
            let text = que.iter().fold(String::new(), |acc, x| acc + x);
            pb.set_message(format!("{ttft}ms {toks:.1}tok/s {text}"));
            pb.tick();

            if ts - begin > chrono::Duration::seconds(config.duration as i64) {
                pb.finish_with_message("finished");
                return Ok(());
            }

            last_success = ts;
        }
        ite += 1;
    }
    pb.finish_with_message("finished");
    Ok(())
}
