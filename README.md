# Warframe syndicate offerings price scraper

There's also a [javascript version](https://gist.github.com/asasinmode/bc9135c9a523ac63736b20e43ff67732)

## Prerequisites

This is a [Rust]([https://nodejs.org/en](https://www.rust-lang.org/)) script for getting the prices of tradeable syndicate offerings in Warframe. For the script to run, cargo and its related binaries have to be installed.

To run it, create a new folder then inside it save the [cargo.toml](#file-cargo-toml) and create another `src` folder and save the [main.rs](#file-main-rs) inside of that. The folder structure should look something like this

```sh
offerings-scraper
├── Cargo.toml         # dependencies
├── src/               # source code folder
│   ├── main.rs        # the code
```

## running it

The script fetches the selected syndicate wiki page and scrapes its offerings, then one by one checks their price on [warframe.market](https://warframe.market) and outputs the results sorted by their lowest price available. To run it, execute the following in your terminal (make sure you are in the folder where the script is located, when ran for the first time it's going to take a bit longer to download the dependencies and set up the project)

```sh
cargo run
```

The script will prompt you to choose a syndicate

```
offerings-scraper ❯ cargo run
? Select a syndicate ›
❯ Steel_Meridian
  Arbiters of Hexis
  Cephalon Suda
  The Perrin Sequence
  Red Veil
  New Loka
```

then start processing with the output being something along the lines of

```
offerings-scraper ❯ cargo run
✔ Select a syndicate · Arbiters of Hexis
Decurion Receiver 1/67
Velocitus Barrel 2/67
Corvas Receiver 3/67
...
Synoid Simulor 66/67
Synoid Heliocor 67/67
--------------------
The Relentless Lost: 5, 7, 9, 10, 10
Entropy Spike: 6, 8, 9, 9, 10
Entropy Detonation: 7, 8, 9, 9, 10
...
Synoid Simulor: 30, 30, 30, 33, 35
Synoid Heliocor: 35, 35, 35, 37, 40
```

The output is split into 2 parts. The first one logs the progress (and any errors that might arise). The second part lists the offerings along with their **lowest 5 prices, listed by _users online in game_**. They are sorted with the most expensive ones are at the bottom.

> [!NOTE]  
> The script fetches 2 offerings per second to respect the [warframe.market's TOS](https://warframe.market/tos). Additionally, the script creates a `.asasinmode_offerings_cache.json` file containing fetched data to be reused in case of multiple script runs. The cache is invalidated after 5 minutes.

### additional info

In case you run into any issues with the script, feel free to leave comments in the gist or message me in game or on discord (discord/ign `asasinmode`)
