use std::io::Write;
use tld_extract::TLDExtract;
use argh::FromArgs;

#[derive(FromArgs)]
/// Reach new heights.
struct Config {
    /// print format json
    #[argh(switch, short = 'j')]
    json: bool,

    /// target
    #[argh(option, short = 't')]
    target: Option<String>,

    /// interactive mode
    #[argh(switch, short = 'i')]
    interactive: bool,

    /// disable private domains
    #[argh(switch)]
    disable_private_domains: bool,
}

fn main() -> Result<(), tld_extract::TLDExtractError> {
    let config: Config = argh::from_env();
    let source = tld_extract::Source::Snapshot;
    let suffix = tld_extract::SuffixList::new(source, config.disable_private_domains, None);
    let mut extract = TLDExtract::new(suffix, true)?;
    if let Some(target) = config.target {
        let e = extract.extract(&target)?;
        if config.json {
            let s = serde_json::to_string_pretty(&e).unwrap();
            println!("{s:}");
        } else {
            println!("{e:?}");
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
