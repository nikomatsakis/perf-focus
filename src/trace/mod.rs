use std::io::{BufRead, stdin};
// use regex::Regex;

#[cfg(test)] mod test;

pub fn each_trace<B,F>(stdin: B, mut callback: F)
    where B: BufRead, F: FnMut(&[String])
{
    let mut trigger = |mut frames: Vec<String>| -> Vec<String> {
        if !frames.is_empty() {
            // frames is a vector containing one sample from perf,
            // including the header line:
            //
            // rustc 18883 2323302.039150: cycles:
            //     7f82e6dee178 je_arena_salloc (/some/path.so)
            //     ...

            {
                // First, extract the name of the process
                // let mut header_words = frames[0].split(char::is_whitespace);
                // let process_name = header_words.next().unwrap();
                // let pid = header_words.next().unwrap();

                // Next, create a secondary vector containing just the
                // callstack. Put this in order from top to bottom
                // (reverse of perf), since that's what the matching code
                // expects. (Arguably we should rewrite the matching
                // code.)
                let mut stack = vec![];
                for frame in frames[1..].iter().rev() {
                    let mut words = frame.trim().split(char::is_whitespace);
                    let fn_name = words.nth(1).unwrap().to_string();
                    stack.push(fn_name);
                }

                callback(&stack);
            }

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

    let mut frames = vec![];
    for line in stdin.lines() {
        let line = line.unwrap();

        // comment
        if line.starts_with('#') {
            continue;
        }

        // empty line.
        if line.trim().is_empty() {
            frames = trigger(frames);
            continue;
        }

        // header line like `rustc 18883 2323302.039150: cycles:`:
        if !line.starts_with(char::is_whitespace) {
            frames = trigger(frames);

            frames.push(line);
            continue;
        }

        // data like `7f82e6dee178 je_arena_salloc (/some/path.so)`
        assert!(!frames.is_empty()); // should have the head line in it
        frames.push(line);
    }

    trigger(frames);
}

