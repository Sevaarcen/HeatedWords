# Heated Words
A Rust program for turning a search term into a wordlist for dictionary-based password attacks. Using hashcat rules with the wordlist is recommended.

Use my other program "[Common Words Generator](https://github.com/Sevaarcen/CommonWordsGenerator)" to build blacklists for the search engines.

## Features
* Concurrent IO for network operations
* Save settings in between runs via the Configuration file (config.toml)
* Extensive Command Line support to change the behavior of the individual execution
* Built in modular support for nearly any type of post-processing  

## Upcoming features
* Amazon Alexa support
* Build in HashCat rule generator
* Higher accuracy and precision
  * Recursive search engine
  * Neural network
  * Dictionary filters
