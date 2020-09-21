use std::fmt::{self, Display};
use std::io::Write;
use std::str::from_utf8_unchecked;
use std::time::Instant;

use anyhow::Result;
use clap::{App, Arg};
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;

const BENCH_SIZE: usize = 1_000_000;
const BENCH_ITER: usize = 100;

fn main() -> Result<()> {
    let matches = App::new("itoa-example")
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .takes_value(true)
                .help("RNG seed"),
        )
        .get_matches();

    let mut rng = if let Some(seed_str) = matches.value_of("seed") {
        let seed: u64 = seed_str.parse()?;
        XorShiftRng::seed_from_u64(seed)
    } else {
        XorShiftRng::from_entropy()
    };

    println!("Digits\tSimpleAvg\tSimpleMin\tSimpleMax\tStdAvg\tStdMin\tStdMax");
    for digits in 1..20 {
        bench_for_digits(&mut rng, digits);
    }

    Ok(())
}

fn bench_for_digits(rng: &mut impl Rng, digits: u32) {
    let value_min = 10u64.pow(digits - 1);

    eprintln!(
        "For {} digits ({} ~ {}):",
        digits,
        value_min,
        value_min * 10 - 1
    );

    let mut simple_times = Vec::<f64>::new();
    let mut std_times = Vec::<f64>::new();

    for _ in 0..BENCH_ITER {
        let mut values = Vec::with_capacity(BENCH_SIZE);
        for _ in 0..BENCH_SIZE {
            values.push(rng.gen_range(value_min, value_min * 10));
        }

        let mut w_simple = Vec::<u8>::with_capacity(21 * BENCH_SIZE);
        let start = Instant::now();
        for v in values.iter().copied() {
            write!(w_simple, "{},", SimpleDisplay(v)).unwrap();
        }
        simple_times.push(start.elapsed().as_secs_f64());

        let mut w_std = Vec::<u8>::with_capacity(21 * BENCH_SIZE);
        let start = Instant::now();
        for v in values.iter().copied() {
            write!(w_std, "{},", v).unwrap();
        }
        std_times.push(start.elapsed().as_secs_f64());

        assert_eq!(w_simple, w_std);
    }

    let simple_stats = stats(&simple_times);
    let std_stats = stats(&std_times);

    eprintln!(
        "    Simple: avg = {:.3}s, min = {:.3}s, max = {:.3}s",
        simple_stats.avg, simple_stats.min, simple_stats.max
    );
    eprintln!(
        "    Std:    avg = {:.3}s, min = {:.3}s, max = {:.3}s",
        std_stats.avg, std_stats.min, std_stats.max
    );

    println!(
        "{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
        digits,
        simple_stats.avg,
        simple_stats.min,
        simple_stats.max,
        std_stats.avg,
        std_stats.min,
        std_stats.max
    );
}

/// 素朴なitoa実装のラッパー型
#[derive(Debug, Clone, Copy)]
struct SimpleDisplay(u64);

impl Display for SimpleDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut n = self.0;

        let mut buf = *b"18446744073709551615";
        let mut cur = buf.len();

        while {
            // do-while と等価なイディオム
            cur -= 1;
            let m = n % 10;
            n = n / 10;
            buf[cur] = (m as u8) + b'0';

            n > 0
        } {}

        unsafe {
            let buf_slice = from_utf8_unchecked(&buf[cur..]);
            f.pad_integral(true, "", buf_slice)
        }
    }
}

fn stats(vs: &[f64]) -> Stats {
    Stats {
        min: vs
            .iter()
            .copied()
            .min_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap(),
        max: vs
            .iter()
            .copied()
            .max_by(|x, y| x.partial_cmp(y).unwrap())
            .unwrap(),
        avg: vs.iter().copied().sum::<f64>() / (BENCH_ITER as f64),
    }
}

#[derive(Debug, Clone, Copy)]
struct Stats {
    avg: f64,
    min: f64,
    max: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_display() {
        let vals = [0, 1, 9, 10, 11, 18446744073709551615];

        for val in vals.iter().copied() {
            let to_be = format!("{}", val);
            let actual = format!("{}", SimpleDisplay(val));
            assert_eq!(actual, to_be);
        }
    }
}
