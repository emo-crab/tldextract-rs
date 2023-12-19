use std::io::Write;
use crossterm::style::Stylize;
use tldextract_cli::Config;
use tldextract_rs::{TLDExtract};

fn main() -> Result<(), tldextract_rs::TLDExtractError> {
    let config: Config = argh::from_env();
    let source = config.clone().into();
    let suffix = tldextract_rs::SuffixList::new(source, config.disable_private_domains, None);
    let mut extract = TLDExtract::new(suffix, true)?;
    let targets = config.targets()?;
    if !targets.is_empty() {
        for target in targets {
            let e = extract.extract(&target)?;
            if config.json {
                let s = serde_json::to_string(&e).unwrap();
                println!("{s:}");
            } else {
                if let Some(f) = &config.filter {
                    let value = match f.to_lowercase().as_str() {
                        "subdomain" => e.subdomain,
                        "domain" => e.domain,
                        "suffix" => e.suffix,
                        "registered_domain" => e.registered_domain,
                        _ => { None }
                    };
                    println!("{}", value.unwrap_or_default());
                } else {
                    print!("[ {} |", e.suffix.unwrap_or("N/A".to_string()).red());
                    print!(" {}", e.domain.unwrap_or("N/A".to_string()).green());
                    print!(" | {} | ", e.registered_domain.unwrap_or("N/A".to_string()));
                    if let Some(sub) = e.subdomain {
                        print!("{}", sub.green());
                    }
                    println!(" ]");
                }
            }
        }
    }
    if config.interactive {
        // println!("Please enter some text: ");
        loop {
            print!("Please enter some text(exit to exit):");
            let mut user_input = String::new();
            let _ = std::io::stdout().flush();
            std::io::stdin().read_line(&mut user_input).expect("Did not enter a correct string");
            if let Some('\n') = user_input.chars().next_back() {
                user_input.pop();
            }
            if let Some('\r') = user_input.chars().next_back() {
                user_input.pop();
            }
            if user_input == "exit" {
                break;
            }
            let e = extract.extract(&user_input)?;
            if config.json {
                let s = serde_json::to_string_pretty(&e).unwrap();
                println!("{s:}");
            } else {
                println!("{e:?}");
            }
        }
    }
    Ok(())
}
