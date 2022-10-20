extern crate skim;
use skim::prelude::*;
use std::{io::Cursor, collections::HashMap};

mod runner;
use runner::Runner;

pub fn main() {
    let x = select_new_runner();
    println!("{:?}", x);
}

fn select_new_runner() -> Option<Runner> {
    let options = SkimOptionsBuilder::default()
        .preview(Some(""))
        .preview_window(Some(""))
        .build()
        .unwrap();

    let runners = generate_runner_list();

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();

    for r in runners.values() {
        tx.send(Arc::new(r.clone())).unwrap();
    }

    drop(tx);

    let item_reader = SkimItemReader::default();


    let r = Skim::run_with(&options, Some(rx));

    if r.is_none() {
        println!("internal runquick error :(");
        return None
    }

    let result = r.unwrap();

    if result.final_event == Event::EvActAbort {
        println!("Nothing Selected...");
        return None
    }

    if result.selected_items.len() != 1 {
        unreachable!()
    }

    let key = result.selected_items[0].output();
    println!("Selected: {}", key);
    match runners.get(&key.to_string()) {
        Some(rnr) => return Some(rnr.to_owned()),
        None => return None,
    }
}

fn generate_runner_list() -> HashMap<String, Runner> {
    //TODO: do this properly, loading from /etc/runquick
    //and ~/.config/runquick/runners.ini

    let mut runners: HashMap<String, Runner> = HashMap::new();

    runners.insert("rust".to_string(),Runner::new("rust", "cmd=cargo run"));
    runners.insert("rust test".to_string(),Runner::new("rust test", "cmd=cargo run"));
    runners.insert("haskell".to_string(),Runner::new("haskell", "cmd=ghci"));

    runners
}