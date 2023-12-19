use tldextract_rs::TLDExtract;

fn main() {
  let source = tldextract_rs::Source::Snapshot;
  let suffix = tldextract_rs::SuffixList::new(source, false, None);
  let mut extract = TLDExtract::new(suffix, true).unwrap();
  let e = extract.extract("  mirrors.tuna.tsinghua.edu.cn");
  println!("{e:#?}");
}
// cargo doc --no-default-features --open --no-deps --package tldextract --offline --lib --examples
