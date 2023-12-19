//! Use the public suffix list to resolve the top-level domain name
//!
//! ## Examples
//! ```rust,no_run
//! use tldextract_rs::TLDExtract;
//! let source = tldextract_rs::Source::Snapshot;
//! let suffix = tldextract_rs::SuffixList::new(source, false, None);
//! let mut extract = TLDExtract::new(suffix, true).unwrap();
//! let e = extract.extract("  www.setup.zip");
//! println!("{:#?}", e);
//! ```
#![warn(missing_docs)]

pub use crate::suffix_list::{Source, SuffixList};
pub use error::{Result, TLDExtractError};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;

mod error;
mod snapshot;
mod suffix_list;

/// TLDTrieTree
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone)]
pub struct TLDTrieTree {
  // 节点
  node: HashMap<String, TLDTrieTree>,
  // 是否可以为顶级域名
  end: bool,
}

impl TLDTrieTree {
  /// Insert TLDTrieTree Construction Data
  #[inline]
  fn insert(&mut self, keys: Vec<&str>) {
    let keys_len = keys.len();
    let mut current_node = &mut self.node;
    for (index, mut key) in keys.clone().into_iter().enumerate() {
      let mut is_exclude = false;
      // 以!开头的需要排除掉
      if index == keys_len - 1 && key.starts_with('!') {
        key = &key[1..];
        is_exclude = true;
      }
      // 获取下一个节点，没有就插入默认节点
      let next_node = current_node.entry(key.to_string()).or_insert(TLDTrieTree {
        node: Default::default(),
        end: false,
      });
      // 当这是最后一个节点，设置可以为顶级域名
      if !is_exclude && (index == keys_len - 1)
                // 最后一个为*的，节点可以为顶级域名
                || (key != "*" && index == keys_len - 2 && keys[index + 1] == "*")
      {
        next_node.end = true;
      }
      current_node = &mut next_node.node;
    }
  }
  /// Search tree, return the maximum path searched
  #[inline]
  fn search(&self, keys: &[String]) -> Vec<Suffix> {
    let mut suffix_list = Vec::new();
    let mut current_node = &self.node;
    for key in keys.iter() {
      match current_node.get(key) {
        Some(next_node) => {
          suffix_list.push(Suffix {
            suffix: key.to_string(),
            end: next_node.end,
          });
          current_node = &next_node.node;
        }
        None => {
          if let Some(next_node) = current_node.get("*") {
            suffix_list.push(Suffix {
              suffix: key.to_string(),
              end: next_node.end,
            });
          }
          break;
        }
      }
    }
    suffix_list
  }
}

#[derive(Debug)]
struct Suffix {
  suffix: String,
  end: bool,
}

/// ExtractResult
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Clone, Debug, Default)]
pub struct ExtractResult {
  /// The "mirrors.tuna" part of "mirrors.tuna.tsinghua.edu.cn"
  pub subdomain: Option<String>,
  /// The "tsinghua" part of "mirrors.tuna.tsinghua.edu.cn"
  pub domain: Option<String>,
  /// The "edu.cn" part of "mirrors.tuna.tsinghua.edu.cn"
  pub suffix: Option<String>,
  /// The "tsinghua.edu.cn" part of "mirrors.tuna.tsinghua.edu.cn"
  pub registered_domain: Option<String>,
}

/// TLDExtract
#[derive(Debug)]
pub struct TLDExtract {
  suffix_list: SuffixList,
  tld_trie: TLDTrieTree,
  domain_to_unicode: bool,
}

impl Default for TLDExtract {
  fn default() -> Self {
    let mut suffix = SuffixList::default();
    let trie = suffix.build().expect("default trie build error");
    TLDExtract {
      suffix_list: suffix.clone(),
      tld_trie: trie,
      domain_to_unicode: true,
    }
  }
}

impl TLDExtract {
  /// Creates a new TLDExtract from suffix
  #[inline]
  pub fn new(suffix: SuffixList, domain_to_unicode: bool) -> Result<Self> {
    let mut new_suffix = suffix;
    let trie = new_suffix.build()?;
    Ok(TLDExtract {
      suffix_list: new_suffix.clone(),
      tld_trie: trie,
      domain_to_unicode,
    })
  }
  /// update SuffixList
  #[inline]
  pub fn update(&mut self, suffix: Option<SuffixList>) {
    if let Some(new_suffix) = suffix {
      self.suffix_list = new_suffix;
    }
    let backup_tld_trie = self.tld_trie.clone();
    match self.suffix_list.build() {
      Ok(trie) => {
        self.tld_trie = trie;
      }
      Err(_err) => {
        // 恢复之前的数据
        self.tld_trie = backup_tld_trie;
      }
    }
  }
}

///                    hierarchical part
//         ┌───────────────────┴─────────────────────┐
//                     authority               path
//         ┌───────────────┴───────────────┐┌───┴────┐
//   abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1
//   └┬┘   └───────┬───────┘ └────┬────┘ └┬┘           └─────────┬─────────┘ └──┬──┘
// scheme  user information     host     port                  query         fragment
///
impl TLDExtract {
  /// TLDExtract extract
  #[inline]
  pub fn extract(&mut self, target: &str) -> Result<ExtractResult> {
    // 先检查域名是否有效
    let target = match idna::domain_to_ascii(target) {
      Ok(target) => target,
      Err(err) => {
        return Err(TLDExtractError::DomainError(err.to_string()));
      }
    };
    let target = target
      .trim_matches(|ch: char| ch.is_whitespace() || ch <= ' ' || ch.is_control())
      .to_string();
    for (index, ch) in target.chars().enumerate() {
      if !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-'
        || ((index == 0 || index == target.len() - 1) && ch == '-')
      {
        return Err(TLDExtractError::DomainError(format!("char:{ch}")));
      }
    }
    // target.chars().map(|ch| ch.is_alphanumeric());
    let keys: Vec<String> = target.rsplit('.').map(|s| s.to_string()).collect();
    let mut extract_result = ExtractResult::default();
    if self.suffix_list.is_expired() {
      self.update(None);
    }
    let mut suffix_list = self.tld_trie.search(&keys);
    let rev_key: Vec<String> = keys.clone().into_iter().rev().collect();
    let rev_key = rev_key.as_slice();
    let mut sl = Vec::new();
    while let Some(s) = suffix_list.pop() {
      if s.end {
        sl.push(s.suffix);
        while let Some(s) = suffix_list.pop() {
          sl.push(s.suffix);
        }
      }
    }
    if !sl.is_empty() {
      let suffix = self.domain_to_unicode(sl.join("."));
      extract_result.suffix = Some(suffix);
    }
    // 域名本身就是顶级域名
    if keys.len() == sl.len() {
      return Ok(extract_result);
    }
    // 顶级域名的分界线索引
    let index = rev_key.len() - sl.len() - 1;
    let domain = self.domain_to_unicode(rev_key.index(index).to_string());
    if !domain.is_empty() {
      extract_result.domain = Some(domain);
    }
    let subdomain = self.domain_to_unicode(rev_key[..index].join("."));
    let registered_domain = self.domain_to_unicode(rev_key[index..].join("."));
    if !subdomain.is_empty() {
      extract_result.subdomain = Some(subdomain);
    }
    if !sl.is_empty() {
      extract_result.registered_domain = Some(registered_domain);
    }
    Ok(extract_result)
  }
  /// If domain name conversion to PunyCode is enabled, the domain name will be re encoded
  fn domain_to_unicode(&self, mut domain: String) -> String {
    if self.domain_to_unicode {
      let (unicode, err) = idna::domain_to_unicode(&domain);
      if err.is_ok() && !unicode.is_empty() {
        domain = unicode;
      }
    }
    domain
  }
}
