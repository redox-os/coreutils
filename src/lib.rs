use std::time::{SystemTime, UNIX_EPOCH};

pub mod columns {
    extern crate termion;
    extern crate extra;
    use std::cmp::max;
    use std::cmp::min;
    use std::io::{stdout, stderr, Write, BufWriter};
    use self::extra::option::OptionalExt;

    /// Prints a vector of strings in a table-like way to the terminal
    pub fn print_columns(words: Vec<String>) {
        let stdout = stdout();
        let mut stdout = BufWriter::new(stdout.lock());
        let mut stderr = stderr();

        let terminal_size = termion::terminal_size().unwrap().0 as usize;
        let columned = make_columns(words, terminal_size);
        for i in 0..columned[0].len() {
            for j in 0..columned.len() {
                if i < columned[j].len() {
                    stdout.write(columned[j][i].as_bytes()).try(&mut stderr);
                }
            }
            stdout.write("\n".as_bytes()).try(&mut stderr);
        }
        stdout.flush().try(&mut stderr);
    }


    /// Turns the list of words into this: line<columns<word>>
    fn make_columns(mut words: Vec<String>, terminal_width: usize) -> Vec<Vec<String>> {

        let word_lengths: Vec<usize> =
            words.iter().map(|x: &String| -> usize {(&x).len() + 2}).collect();

        let columns_amt = bin_search( word_lengths.iter().fold(0   , |x, y| {min(x, *y)})
                                    , word_lengths.iter().fold(1000, |x, y| {max(x, *y)})
                                    , &word_lengths
                                    , terminal_width);

        let longest_words: Vec<usize> =
            split_into_columns( &word_lengths
                              , columns_amt
                              , (words.len() / columns_amt) + 1
                              ).iter().map(longest_word).collect();

        let mut words_with_space: Vec<String> = Vec::new();
        let lines_amt = (words.len() / columns_amt) + 1;
        let mut longest_words_rep = Vec::new();
        for longest_word in longest_words {
            for _ in 0..lines_amt {
                longest_words_rep.push(longest_word.clone());
            }
        }

        for i in 0..words.len() {

            let whitespace = " ".repeat(longest_words_rep[i] - words[i].len());
            words[i].push_str(whitespace.as_str());
            words_with_space.push(words[i].clone());
        }

        split_into_columns::<String>(&words_with_space, columns_amt, (words.len() / columns_amt) + 1)
    }

    /// Binary search to find the perfect amounth of columns efficiently
    fn bin_search(min: usize, max: usize, words: &Vec<usize>, terminal_width: usize) -> usize {
        let diff = min as isize - max as isize;
        let fits = try_rows(words, min + (max - min) / 2, terminal_width);
        if  diff == -1 || diff == 0 || diff == 1{
            return min;
        } else if fits {
            return bin_search(min + (max - min) / 2, max, words, terminal_width);
        } else {
            return bin_search(min, min + (max - min) / 2, words, terminal_width);
        }
    }

    /// Find the length of the longest word in a list
    fn longest_word(words: &Vec<usize>) -> usize {
        words.iter().fold(0, |x, y| {max(x, *y)})
    }

    /// splits a vector of cloneables into a vector of vectors of cloneables where lines_amt
    /// is the length of the outer vector and columns the max len of the inner vector
    fn split_into_columns<T: Clone>(words: &Vec<T>, columns: usize, lines_amt: usize) -> Vec<Vec<T>> {
        assert!(words.len() <= columns * lines_amt);
        let mut outputs: Vec<Vec<T>> = Vec::new();
        for _ in 0..columns {
            outputs.push(Vec::new())
        }
        let mut i = 0;
        'outer: for output in &mut outputs {
            let mut j = 0;
            while j < lines_amt {
                if i >= words.len() {
                    break 'outer;
                }
                output.push(words.get(i).unwrap().clone());
                i += 1;
                j += 1;
            }
        }
        outputs
    }

    /// Tests wheter a vector of words, split into a certain number of columns fits into given width
    fn try_rows(input: &Vec<usize>, columns: usize, width: usize) -> bool {

        let lines_amt = (input.len() / columns) + 1;

        let sum_line_widths = |widths_list: Vec<usize>| -> usize {
            widths_list.iter().fold(0, |x, y| { x + *y})
        };

        sum_line_widths(
            split_into_columns(input, columns, lines_amt).iter().map(
                longest_word
            ).collect()
        ) <= width
    }
}

pub fn format_system_time(time: SystemTime) -> String {
    let tz_offset = 0; //TODO Apply timezone offset
    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => format_time(duration.as_secs() as i64, tz_offset),
        Err(_) => "duration since epoch err".to_string(),
    }
}

// Sweet algorithm from http://ptspts.blogspot.com/2009/11/how-to-convert-unix-timestamp-to-civil.html
// TODO: Apply timezone offset
pub fn get_time_tuple(mut ts: i64, tz_offset: i64) -> (i64, i64, i64, i64, i64, i64) {
    ts += tz_offset * 3600;
    let s = ts % 86400;
    ts /= 86400;
    let h = s / 3600;
    let m = s / 60 % 60;
    let s = s % 60;
    let x = (ts * 4 + 102032) / 146097 + 15;
    let b = ts + 2442113 + x - (x / 4);
    let mut c = (b * 20 - 2442) / 7305;
    let d = b - 365 * c - c / 4;
    let mut e = d * 1000 / 30601;
    let f = d - e * 30 - e * 601 / 1000;
    if e < 14 {
        c -= 4716;
        e -= 1;
    } else {
        c -= 4715;
        e -= 13;
    }
    (c, e, f, h, m, s)
}

pub fn format_time(ts: i64, tz_offset: i64) -> String {
    let (c, e, f, h, m, s) = get_time_tuple(ts, tz_offset);
    format!("{:>04}-{:>02}-{:>02} {:>02}:{:>02}:{:>02}", c, e, f, h, m, s)
}

pub fn to_human_readable_string(size: u64) -> String {
    if size < 1024 {
        return format!("{}", size);
    }

    static UNITS: [&'static str; 7] = ["", "K", "M", "G", "T", "P", "E"];

    let sizef = size as f64;
    let digit_groups = (sizef.log10() / 1024f64.log10()) as i32;
    format!("{:.1}{}",
            sizef / 1024f64.powf(digit_groups as f64),
            UNITS[digit_groups as usize])
}
