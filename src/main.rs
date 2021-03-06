extern crate appdirs;
extern crate clap;

use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::panic;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use appdirs::user_data_dir;
use clap::{App, Arg, ArgMatches, SubCommand};

fn getppid() -> i32 {
    #[link(name = "process", kind = "static")]
    extern "C" {
        fn getppid() -> i32;
    }
    unsafe { getppid() }
}

struct Time {
    label: String,
    lap: f64,
    split: f64,
}

struct StoredTime {
    label: String,
    nanoseconds: u64,
}

struct Stopwatch<'a> {
    path: &'a Path,
}

impl<'a> Stopwatch<'a> {
    fn start(&self, time: u64) -> Result<(), &str> {
        let mut file = match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.path)
        {
            Ok(file) => file,
            Err(_) => return Err("Stopwatch already running"),
        };
        file.write_all(format!("0: {}\n", time.to_string()).as_bytes())
            .unwrap();
        Ok(())
    }

    fn stop(&self) -> Result<(), &str> {
        match fs::remove_file(self.path) {
            Ok(_) => Ok(()),
            Err(_) => Err("No running stopwatch"),
        }
    }

    fn times(&self) -> Result<Vec<Time>, &str> {
        let mut times = Vec::new();
        let stored_times = self.stored_times()?;
        let start_time = stored_times.first().unwrap().nanoseconds;
        let mut prev_time = start_time;

        for time in stored_times.into_iter().skip(1) {
            let split_time = ((time.nanoseconds - start_time) as f64) / 1e9;
            let lap_time = ((time.nanoseconds - prev_time) as f64) / 1e9;
            prev_time = time.nanoseconds;
            times.push(Time {
                label: time.label,
                split: split_time,
                lap: lap_time,
            });
        }

        Ok(times)
    }

    fn elapsed(&self, time: u64) -> Result<Time, &str> {
        let times = self.stored_times()?;
        let start = times.first().unwrap().nanoseconds;
        let last = times.last().unwrap().nanoseconds;
        let split_time = ((time - start) as f64) / 1e9;
        let lap_time = ((time - last) as f64) / 1e9;
        let label = times.len().to_string();
        Ok(Time {
            label: label,
            split: split_time,
            lap: lap_time,
        })
    }

    fn record(&self, time: u64, label: Option<&str>) -> Result<(), &str> {
        let mut file = match OpenOptions::new().append(true).open(self.path) {
            Ok(file) => file,
            Err(_) => return Err("No running stopwatch"),
        };
        file.write_all(
            format!(
                "{}: {}\n",
                label
                    .unwrap_or(&(self.times().unwrap().len() + 1).to_string()),
                time.to_string()
            )
            .as_bytes(),
        )
        .unwrap();
        Ok(())
    }

    fn stored_times(&self) -> Result<Vec<StoredTime>, &str> {
        let file = match File::open(self.path) {
            Ok(file) => file,
            Err(_) => return Err("No running stopwatch"),
        };
        Ok(BufReader::new(file)
            .lines()
            .map(|line| {
                let line = line.unwrap().to_string();
                let mut parts = line.split(": ");
                let label = parts.next().unwrap().to_string();
                let nanoseconds =
                    parts.next().unwrap().to_string().parse::<u64>().unwrap();
                StoredTime {
                    label: label,
                    nanoseconds: nanoseconds,
                }
            })
            .collect())
    }
}

fn process_subcommand(
    subcommand: (&str, Option<&ArgMatches>),
    stopwatch: Stopwatch,
    time: u64,
) -> Result<Option<String>, String> {
    match subcommand {
        ("start", _) => match stopwatch.start(time) {
            Ok(_) => Ok(None),
            Err(e) => Err(e.to_string()),
        },
        ("stop", _) => match stopwatch.stop() {
            Ok(_) => Ok(None),
            Err(e) => Err(e.to_string()),
        },
        ("record", Some(sub_m)) => {
            let label = sub_m.value_of("label");
            match stopwatch.record(time, label) {
                Ok(_) => Ok(None),
                Err(e) => Err(e.to_string()),
            }
        }
        ("split", Some(sub_m)) => {
            let label = sub_m.value_of("label");
            match stopwatch.record(time, label) {
                Ok(_) => Ok(Some(format!(
                    "{}",
                    stopwatch.times().unwrap().last().unwrap().split
                ))),
                Err(e) => Err(e.to_string()),
            }
        }
        ("lap", Some(sub_m)) => {
            let label = sub_m.value_of("label");
            match stopwatch.record(time, label) {
                Ok(_) => Ok(Some(format!(
                    "{}",
                    stopwatch.times().unwrap().last().unwrap().lap
                ))),
                Err(e) => Err(e.to_string()),
            }
        }
        ("times", _) => {
            let times = stopwatch.times()?;
            let mut times_table = String::new();
            let label_length =
                times.iter().map(|t| t.label.len()).max().unwrap();
            let split_length = times
                .iter()
                .map(|t| t.split.to_string().len())
                .max()
                .unwrap();
            let lap_length =
                times.iter().map(|t| t.lap.to_string().len()).max().unwrap();
            for time in times.iter() {
                times_table.push_str(&format!(
                    "{0:1$} {2:3$} {4:5$}\n",
                    time.label,
                    label_length,
                    time.split,
                    split_length,
                    time.lap,
                    lap_length
                ));
            }
            times_table.pop(); // remove newline
            Ok(Some(times_table))
        }
        ("elapsed", Some(sub_m)) => {
            let label = sub_m.value_of("label");
            let time = match label {
                Some(label) => {
                    let times = stopwatch.times()?;
                    match times.into_iter().find(|t| t.label == label) {
                        Some(time) => time,
                        None => {
                            return Err(format!("Invalid label {}", label))
                        }
                    }
                }
                None => match stopwatch.elapsed(time) {
                    Ok(time) => time,
                    Err(e) => return Err(e.to_string()),
                },
            };
            match sub_m.is_present("lap") {
                true => Ok(Some(format!("{}", time.lap))),
                false => Ok(Some(format!("{}", time.split))),
            }
        }
        _ => match stopwatch.elapsed(time) {
            Ok(time) => match stopwatch.stop() {
                Ok(_) => Ok(Some(format!("Time elapsed: {}s", time.split))),
                Err(e) => Err(e.to_string()),
            },
            Err(_) => match stopwatch.start(time) {
                Ok(_) => Ok(None),
                Err(e) => Err(e.to_string()),
            },
        },
    }
}

fn main() {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    panic::set_hook(Box::new(|panic_info| {
        let s = panic_info.payload().downcast_ref::<String>().unwrap();
        eprintln!("{}: {}", env::args().next().unwrap(), s);
    }));

    let dir = user_data_dir(Some("sw"), None, false).unwrap();

    fs::create_dir_all(&dir).expect(&format!(
        "Could not create directory {}",
        dir.to_str().unwrap()
    ));

    let label_arg = Arg::with_name("label").help("Time label");

    let matches = App::new("sw")
        .about("record elapsed times")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("stdout")
                .short("1")
                .help("Prints to stdout instead of stderr"),
        )
        .subcommand(
            SubCommand::with_name("start").about("Starts the stopwatch"),
        )
        .subcommand(SubCommand::with_name("stop").about("Stops the stopwatch"))
        .subcommand(
            SubCommand::with_name("record")
                .about("Records a time")
                .arg(&label_arg),
        )
        .subcommand(
            SubCommand::with_name("split")
                .about("Records and prints a split time")
                .arg(&label_arg),
        )
        .subcommand(
            SubCommand::with_name("lap")
                .about("Records and prints a lap time")
                .arg(&label_arg),
        )
        .subcommand(
            SubCommand::with_name("elapsed")
                .about("Prints the elapsed time")
                .arg(&label_arg)
                .arg(
                    Arg::with_name("lap")
                        .short("l")
                        .long("lap")
                        .help("Prints the lap time instead of the split time"),
                ),
        )
        .subcommand(
            SubCommand::with_name("times")
                .about("Prints the recorded split and lap times"),
        )
        .get_matches();

    let id = getppid().to_string();
    let path = dir.join(id);

    let stopwatch = Stopwatch { path: &path };

    match process_subcommand(matches.subcommand(), stopwatch, time) {
        Ok(s) => match s {
            Some(s) => match matches.is_present("stdout") {
                true => println!("{}", s),
                false => eprintln!("{}", s),
            },
            None => {}
        },
        Err(e) => panic!("{}", e),
    }
}
