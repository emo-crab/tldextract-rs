use argh::FromArgs;
use std::collections::HashSet;
use std::io::{BufRead, IsTerminal, Read};
use std::path::PathBuf;
use std::str::FromStr;
use tldextract_rs::Source;

#[derive(Clone, FromArgs)]
/// Reach new heights.
pub struct Config {
  /// specific sources(local file path or remote url) to prefix list,(eg. snapshot,remote)
  #[argh(option, short = 's')]
  pub source_uri: Option<String>,

  /// write output in json(lines) format
  #[argh(switch, short = 'j')]
  pub json: bool,

  /// list of sub(domains) to extract (file or stdin)
  #[argh(option, short = 'l')]
  pub list: Option<String>,

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

impl From<Config> for Source {
  fn from(val: Config) -> Self {
    match val.source_uri {
      None => Source::Snapshot,
      Some(s) => Source::from_str(&s).unwrap_or_default(),
    }
  }
}

impl Config {
  pub fn targets(&self) -> Result<HashSet<String>, std::io::Error> {
    match &self.list {
      None => read_from_stdio(),
      Some(s) => {
        if let Ok(l) = read_lines(s) {
          Ok(l.map_while(Result::ok).collect::<HashSet<String>>())
        } else {
          Ok(HashSet::from_iter(vec![s.to_string()]))
        }
      }
    }
  }
}

fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>>
where
  P: AsRef<std::path::Path>,
{
  let file = std::fs::File::open(filename)?;
  Ok(std::io::BufReader::new(file).lines())
}

fn read_from_stdio() -> Result<HashSet<String>, std::io::Error> {
  let (tx, rx) = std::sync::mpsc::channel::<String>();
  let mut stdin = std::io::stdin();
  if stdin.is_terminal() {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "invalid input",
    ));
  }
  std::thread::spawn(move || loop {
    let mut buffer = String::new();
    stdin.read_to_string(&mut buffer).unwrap_or_default();
    if let Err(_err) = tx.send(buffer) {
      break;
    };
  });
  loop {
    match rx.try_recv() {
      Ok(line) => {
        let l = line
          .lines()
          .map(|l| l.to_string())
          .collect::<HashSet<String>>();
        return Ok(l);
      }
      Err(std::sync::mpsc::TryRecvError::Empty) => {}
      Err(std::sync::mpsc::TryRecvError::Disconnected) => panic!("Channel disconnected"),
    }
    let duration = std::time::Duration::from_millis(1000);
    std::thread::sleep(duration);
  }
}
