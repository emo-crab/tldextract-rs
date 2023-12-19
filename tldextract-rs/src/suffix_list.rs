use super::error::Result;
use crate::snapshot::PUBLIC_SUFFIX_LIST;
use crate::TLDExtractError;
use crate::TLDTrieTree;
#[cfg(feature = "reqwest")]
use reqwest::IntoUrl;
use std::collections::HashSet;
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::SystemTime;

const PUBLIC_PRIVATE_SUFFIX_SEPARATOR: &str = "// ===BEGIN PRIVATE DOMAINS===";
#[cfg(feature = "reqwest")]
const PUBLIC_SUFFIX_LIST_URLS: &[&str] = &[
  "https://publicsuffix.org/list/public_suffix_list.dat",
  "https://raw.githubusercontent.com/publicsuffix/list/master/public_suffix_list.dat",
];

/// Where to read data as a prefix list
#[derive(Debug, Clone, Default)]
pub enum Source {
  /// Read from text
  Text(String),
  /// Snapshot
  #[default]
  Snapshot,
  /// Read from file
  Local(PathBuf),
  /// Read from remote URL，NONE default: PUBLIC_SUFFIX_LIST_URLS
  #[cfg(feature = "reqwest")]
  Remote(Option<reqwest::Url>),
}

impl FromStr for Source {
  type Err = TLDExtractError;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    match s {
      "snapshot" => Ok(Source::Snapshot),
      #[cfg(feature = "reqwest")]
      "remote" => Ok(Source::Remote(None)),
      _ => {
        if let Ok(p) = <&str as TryInto<PathBuf>>::try_into(s) {
          return if p.exists() {
            Ok(Source::Local(p))
          } else {
            Err(TLDExtractError::Io(std::io::Error::new(
              std::io::ErrorKind::NotFound,
              "file does not exist",
            )))
          };
        };
        #[cfg(feature = "reqwest")]
        if let Ok(u) = s.try_into() {
          return Ok(Source::Remote(Some(u)));
        }
        Ok(Source::Snapshot)
      }
    }
  }
}

/// Mainly implementing the resolution and classification of domain names
#[derive(Debug, Default, Clone)]
pub struct SuffixList {
  /// Source [Source]
  pub source: Source,
  /// Extra source [Source]
  pub extra: Option<Source>,
  /// Custom append public_suffixes
  pub public_suffixes: HashSet<String>,
  /// Custom append private_suffixes
  pub private_suffixes: HashSet<String>,
  /// Whether to disable private domains
  pub disable_private_domains: bool,
  /// Expiration time
  pub expire: Option<std::time::Duration>,
  /// Last update time
  pub last_update: std::time::Duration,
}

impl SuffixList {
  /// Creates a new SuffixList with source
  #[inline]
  pub fn new(
    source: Source,
    disable_private_domains: bool,
    expire: Option<std::time::Duration>,
  ) -> Self {
    SuffixList {
      source,
      extra: None,
      public_suffixes: Default::default(),
      private_suffixes: Default::default(),
      disable_private_domains,
      expire,
      last_update: now(),
    }
  }
  /// set disable_private_domains
  #[inline]
  pub fn private_domains(mut self, disable_private_domains: bool) -> Self {
    self.disable_private_domains = disable_private_domains;
    self
  }
  /// set expire
  #[inline]
  pub fn expire(mut self, expire: std::time::Duration) -> Self {
    self.expire = Some(expire);
    self
  }
  /// set source
  #[inline]
  pub fn source(mut self, source: Source) -> Self {
    self.source = source;
    self
  }
  /// set extra source
  #[inline]
  pub fn extra(mut self, extra: Source) -> Self {
    self.extra = Some(extra);
    self
  }
  /// Check if it has expired
  #[inline]
  pub fn is_expired(&self) -> bool {
    match self.expire {
      Some(s) => {
        // 现在时间戳 - 过期时间 > 最后更新时间戳
        now().as_secs() - s.as_secs() > self.last_update.as_secs()
      }
      None => false,
    }
  }
  fn reset(&mut self) {
    self.private_suffixes = HashSet::new();
    self.public_suffixes = HashSet::new();
  }
  fn parse_source(&mut self, source: Source) -> Result<()> {
    let mut is_private_suffix = false;
    let mut tld_lines = Vec::new();
    match source {
      Source::Local(path) => {
        let file = std::fs::File::open(path)?;
        let lines = std::io::BufReader::new(file)
          .lines()
          .map(|l| l.unwrap_or_default());
        tld_lines = lines.collect();
      }
      #[cfg(feature = "reqwest")]
      Source::Remote(u) => match u {
        Some(u) => {
          tld_lines = get_source_from_url(u)?;
        }
        None => {
          let mut tld_err = TLDExtractError::SuffixListError(String::new());
          for u in PUBLIC_SUFFIX_LIST_URLS {
            match get_source_from_url(u.trim()) {
              Ok(lines) => {
                tld_lines = lines;
                break;
              }
              Err(err) => {
                tld_err = err;
              }
            }
          }
          if tld_lines.is_empty() {
            return Err(tld_err);
          }
        }
      },
      Source::Snapshot => {
        let lines = PUBLIC_SUFFIX_LIST.lines().map(|s| s.to_string());
        for line in lines {
          is_private_suffix = self.process_line(line, is_private_suffix);
        }
      }
      Source::Text(text) => {
        let lines = text.lines().map(|s| s.to_string());
        for line in lines {
          is_private_suffix = self.process_line(line, is_private_suffix);
        }
      }
    }
    for line in tld_lines {
      is_private_suffix = self.process_line(line, is_private_suffix);
    }
    Ok(())
  }
  ///  build TLDTrieTree
  #[inline]
  pub fn build(&mut self) -> Result<TLDTrieTree> {
    self.reset();
    self.parse_source(self.source.clone())?;
    if let Some(extra) = self.extra.clone() {
      self.parse_source(extra)?;
    }
    let ttt = self.construct_tree();
    self.last_update = now();
    Ok(ttt)
  }
  // 处理行
  fn process_line(&mut self, raw_line: String, mut is_private_suffix: bool) -> bool {
    // 已经到了私有域名分界线了，而且没有开启私有域名，直接跳过
    if is_private_suffix && self.disable_private_domains {
      return is_private_suffix;
    }
    let line = raw_line.trim_end();
    if !is_private_suffix && PUBLIC_PRIVATE_SUFFIX_SEPARATOR == line {
      is_private_suffix = true;
    }
    if line.is_empty() || line.starts_with("//") {
      return is_private_suffix;
    }
    if let Ok(suffix) = idna::domain_to_ascii(line) {
      if is_private_suffix {
        self.private_suffixes.insert(suffix.clone());
        if suffix != line {
          self.private_suffixes.insert(line.to_string());
        }
      } else {
        self.public_suffixes.insert(suffix.clone());
        if suffix != line {
          self.public_suffixes.insert(suffix);
        }
      }
    }
    is_private_suffix
  }
  // 构造前缀树
  fn construct_tree(&self) -> TLDTrieTree {
    let mut trie_tree = TLDTrieTree {
      node: Default::default(),
      end: false,
    };
    let mut suffix_list = self.public_suffixes.clone();
    if !self.disable_private_domains {
      suffix_list.extend(self.private_suffixes.clone());
    }
    for suffix in suffix_list {
      let sp: Vec<&str> = suffix.rsplit('.').collect();
      trie_tree.insert(sp);
    }
    trie_tree
  }
}

fn now() -> std::time::Duration {
  SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
}

#[cfg(feature = "reqwest")]
fn get_source_from_url<T>(u: T) -> Result<Vec<String>>
where
  T: IntoUrl,
{
  let response = reqwest::blocking::get(u)?;
  let bytes = response.bytes()?;
  let lines = bytes.lines().map(|l| l.unwrap_or_default());
  Ok(lines.collect())
}
