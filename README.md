# Heated Words
A Rust program for turning a search term into a wordlist for dictionary-based password attacks. Using hashcat rules with the wordlist is recommended.

```NOTE: This program doesn't execute Javascript upon retrieving a website and therefore will not build wordlists based on dynamic content```

Use my other program "[Common Words Generator](https://github.com/Sevaarcen/CommonWordsGenerator)" to build blacklists for the search engines.

## Features
* High-performance concurrent IO operations
* Save settings in between runs via the Configuration file (config.toml)
* Config file allows for expansive customization and expandability based on your exact needs!
* Extensive Command Line support to change the behavior for each individual execution
* Built-in modular support for any non-interactive type of post-processing
