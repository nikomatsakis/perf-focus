use std::io::{BufRead, stdin};
// use regex::Regex;

pub fn each_trace<F>(mut callback: F)
    where F: FnMut(&[String])
{
    let mut trigger = |mut frames: Vec<String>| -> Vec<String> {
        if !frames.is_empty() {
            frames.reverse();
            callback(&frames);
            frames.truncate(0);
        }

        frames
    };

    // Note: I used to use regular expressions here, but the perf
    // was not good enough.
    //
    // let head_re = Regex::new(r"^(?P<proc>.*)\s+(?P<pid>\d+)\s").unwrap();
    // let entry_re = Regex::new(r"^\s*[a-f0-9]+ (?P<trace>.*) \(.*\)$").unwrap();
    // let blank_re = Regex::new(r"^\s*$").unwrap();

    let stdin = stdin();
    let stdin = stdin.lock();
    let mut frames = vec![];
    for line in stdin.lines() {
        let line = line.unwrap();

        // comment
        if line.starts_with('#') {
            continue;
        }

        // header line like `rustc 18883 2323302.039150: cycles:`:
        if !line.starts_with(char::is_whitespace) {
            frames = trigger(frames);
            continue;
        }

        let line = line.trim();

        // empty line.
        if line.is_empty() {
            frames = trigger(frames);
            continue;
        }

        // data like 7f82e6dee178 je_arena_salloc (/some/path.so)
        let mut words = line.split(char::is_whitespace);
        let fn_name = words.nth(1).unwrap();
        frames.push(fn_name.to_string());
    }

    trigger(frames);
}

