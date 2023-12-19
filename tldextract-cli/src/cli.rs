use std::collections::HashSet;
use std::io::{BufRead, Read};
use std::path::PathBuf;
use std::str::FromStr;
use argh::FromArgs;
use tldextract_rs::Source;

#[derive(Clone, FromArgs)]
/// Reach new heights.
pub struct Config {
    /// specific sources(local file path or remote url) to prefix list,(eg. snapshot,remote)
    #[argh(option, short = 's')]
    pub source_uri: Option<String>,

    /// write output in jsonl(ines) format
    #[argh(switch, short = 'j')]
    pub json: bool,

    /// list of sub(domains) to extract (file or stdin)
    #[argh(option, short = 'l')]
    pub list: Option<String>,

    /// interactive mode
    #[argh(switch, short = 'i')]
    pub interactive: bool,

    /// disable private domains
    #[argh(switch)]
    pub disable_private_domains: bool,

    /// display filter result by field only (eg. -f suffix,domain,subdomain,registered_domain)
    #[argh(option, short = 'f')]
    pub filter: Option<String>,

    /// file to write output
    #[argh(option, short = 'o')]
    pub output: Option<PathBuf>,
}

impl Into<Source> for Config {
    fn into(self) -> Source {
        match self.source_uri {
            None => { Source::Snapshot }
            Some(s) => {
                Source::from_str(&s).unwrap_or_default()
            }
        }
    }
}

impl Config {
    pub fn targets(&self) -> Result<HashSet<String>, std::io::Error> {
        return match &self.list {
            None => {
                let (tx, rx) = std::sync::mpsc::channel::<String>();
                std::thread::spawn(move || loop {
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer).unwrap_or_default();
                    if let Err(_err) = tx.send(buffer) {
                        break;
                    };
                }
                );
                loop {
                    match rx.try_recv() {
                        Ok(line) => {
                            let l = line.lines().map(|l| l.to_string()).collect::<HashSet<String>>();
                            return Ok(l);
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {}
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("Channel disconnected"),
                    }
                }
            }
            Some(s) => {
                Ok(read_lines(s)?.map_while(Result::ok).collect::<HashSet<String>>())
            }
        };
    }
}

fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>>
    where
        P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(std::io::BufReader::new(file).lines())
}