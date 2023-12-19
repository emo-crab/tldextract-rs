use crossterm::style::Stylize;
use std::fs::File;
use tldextract_cli::Config;
use tldextract_rs::TLDExtract;

fn main() -> Result<(), tldextract_rs::TLDExtractError> {
  let config: Config = argh::from_env();
  let source = config.clone().into();
  let suffix = tldextract_rs::SuffixList::new(source, config.disable_private_domains, None);
  let mut extract = TLDExtract::new(suffix, true)?;
  let targets = config.targets()?;
  let mut result = Vec::new();
  for target in targets {
    if let Ok(e) = extract.extract(&target) {
      result.push(e.clone());
      if config.json {
        let s = serde_json::to_string(&e).unwrap();
        println!("{s:}");
      } else if let Some(f) = &config.filter {
        let value = match f.to_lowercase().as_str() {
          "subdomain" => e.subdomain,
          "domain" => e.domain,
          "suffix" => e.suffix,
          "registered_domain" => e.registered_domain,
          _ => None,
        };
        println!("{}", value.unwrap_or_default());
      } else {
        print!(
          "[ {} |",
          e.subdomain.unwrap_or("N/A".to_string()).dark_magenta()
        );
        print!(
          " {}",
          e.registered_domain.unwrap_or("N/A".to_string()).green()
        );
        print!(" | {} | ", e.domain.unwrap_or("N/A".to_string()).red());
        print!("{}", e.suffix.unwrap_or_default().dark_blue());
        println!(" ]");
      }
    }
  }
  if let Some(o) = config.output {
    let out = File::create(o).expect("Failed to create file");
    serde_json::to_writer(out, &result).expect("Failed to save file");
  }
  Ok(())
}
