## Summary

**tldextract-rs** is a high performance [effective top level domains (eTLD)](https://wiki.mozilla.org/Public_Suffix_List) extraction module that extracts subcomponents from Domain.


## Try the example code

All of the following examples can be found at `examples/example.rs`. To play the demo, run the following command:

```sh
# `git clone` and `cd` to the tldextract-rs repository folder first
cargo run --example=example
```

### Hostname
- Cargo.toml:

```toml
tldextract = { git = "https://github.com/emo-cat/tldextract-rs" }
```

- example code

```rust
use tldextract::TLDExtract;

fn main() {
  let source = tldextract::Source::Hardcode;
  let suffix = tldextract::SuffixList::new(source, false, None);
  let mut extract = TLDExtract::new(suffix, true).unwrap();
  let e = extract.extract("  mirrors.tuna.tsinghua.edu.cn");
  println!("{e:#?}");
}
```

- ExtractResult

```js
Ok(
    ExtractResult {
        subdomain: Some(
            "mirrors.tuna",
        ),
        domain: Some(
            "tsinghua",
        ),
        suffix: Some(
            "edu.cn",
        ),
        registered_domain: Some(
            "tsinghua.edu.cn",
        ),
    },
)
```

## Implementation details

### Why not split on "." and take the last element instead?

Splitting on "." and taking the last element only works for simple eTLDs like `com`, but not more complex ones like `oseto.nagasaki.jp`.

### eTLD tries

**tldextract-rs** stores eTLDs in [compressed tries](https://en.wikipedia.org/wiki/Trie).

Valid eTLDs from the [Mozilla Public Suffix List](http://www.publicsuffix.org) are appended to the compressed trie in reverse-order.

```sh
Given the following eTLDs
au
nsw.edu.au
com.ac
edu.ac
gov.ac

and the example URL host `example.nsw.edu.au`

The compressed trie will be structured as follows:

START
 ╠═ au 🚩 ✅
 ║  ╚═ edu ✅
 ║     ╚═ nsw 🚩 ✅
 ╚═ ac
    ╠═ com 🚩
    ╠═ edu 🚩
    ╚═ gov 🚩

=== Symbol meanings ===
🚩 : path to this node is a valid eTLD
✅ : path to this node found in example URL host `example.nsw.edu.au`
```

The URL host subcomponents are parsed from right-to-left until no more matching nodes can be found. In this example, the path of matching nodes are `au -> edu -> nsw`. Reversing the nodes gives the extracted eTLD `nsw.edu.au`.

## Acknowledgements

- [go-fasttld (Go)](https://github.com/elliotwutingfeng/go-fasttld)
- [tldextract (Python)](https://github.com/john-kurkowski/tldextract)
