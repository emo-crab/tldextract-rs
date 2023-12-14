use tld_extract::TLDExtract;

fn main() {
  let source = tld_extract::Source::Snapshot;
  let suffix = tld_extract::SuffixList::new(source, false, None);
  let mut extract = TLDExtract::new(suffix, true).unwrap();
  let e = extract.extract("  mirrors.tuna.tsinghua.edu.cn");
  println!("{e:#?}");
}
// cargo doc --no-default-features --open --no-deps --package tldextract --offline --lib --examples
