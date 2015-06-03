use std::io::{BufRead, stdin, Stdin};
use regex::Regex;

#[derive(Debug)]
pub struct Trace {
    // each entry is a fn name, low index is bottom of the stack, high
    // index is top of the stack
    pub frames: Vec<String>
}

pub fn each_trace<F>(mut callback: F)
    where F: FnMut(Trace)
{
    let comment_re = Regex::new(r"^#.*").unwrap();
    let head_re = Regex::new(r"^(?P<proc>.*)\s+(?P<pid>\d+)\s").unwrap();
    let entry_re = Regex::new(r"^\s*[a-f0-9]+ (?P<trace>.*) \(.*\)$").unwrap();
    let blank_re = Regex::new(r"^\s*$").unwrap();

    let mut trigger = |mut frames: Vec<String>| -> Vec<String> {
        if !frames.is_empty() {
            frames.reverse();
            callback(Trace { frames: frames });
            vec![]
        } else {
            frames
        }
    };

    let stdin = stdin();
    let stdin = stdin.lock();
    let mut frames = vec![];
    for line in stdin.lines() {
        let line = line.unwrap();

        if comment_re.is_match(&line) {
            continue;
        }

        if head_re.is_match(&line) {
            frames = trigger(frames);
            continue;
        }

        if let Some(caps) = entry_re.captures(&line) {
            frames.push(caps.name("trace").unwrap().to_string());
            continue;
        }

        if blank_re.is_match(&line) {
            frames = trigger(frames);
        }
    }

    trigger(frames);
}

