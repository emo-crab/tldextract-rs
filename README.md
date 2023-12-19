## Summary

**tldextract-rs** is a high performance [effective top level domains (eTLD)](https://wiki.mozilla.org/Public_Suffix_List) extraction module that extracts subcomponents from Domain.

- Using

```bash
Usage: tldextract-cli [-s <source-uri>] [-j] [-l <list>] [--disable-private-domains] [-f <filter>] [-o <output>]

Reach new heights.

Options:
  -s, --source-uri  specific sources(local file path or remote url) to prefix
                    list,(eg. snapshot,remote)
  -j, --json        write output in json(lines) format
  -l, --list        list of sub(domains) to extract (file or stdin)
  --disable-private-domains
                    disable private domains
  -f, --filter      display filter result by field only (eg. -f
                    suffix,domain,subdomain,registered_domain)
  -o, --output      file to write output
  --help            display usage information

```
- example

```bash
➜  tldextract-rs git:(main) ✗ tldextract-cli  -j -l mirrors.tuna.tsinghua.edu.cn
 {"subdomain":"mirrors.tuna","domain":"tsinghua","suffix":"edu.cn","registered_domain":"tsinghua.edu.cn"}
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
