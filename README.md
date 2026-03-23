
# Ficherors

A Rust CLI tool for processing contact files (CSV, TXT, XLSX). It normalises phone numbers to international format, applies column transforms, removes accents, and outputs a clean semicolon-delimited file ready for bulk messaging platforms.

## What it does

- Reads CSV, TXT (`;` `|` `\t` `,` ` ` delimited) and Excel files
- Formats phone numbers to E.164 international format for 20 countries (BR, AR, MX, CO, CL, US, UK...)
- Applies column transforms: `send_date`, `send_hour`, `random_num`, `downcase`, `upcase`, `first_word`, `fixed`, `dynamic`
- Strips accents via Unicode transliteration
- Detects CR/LF line endings
- Validates file structure and phone numbers
- Writes output to a destination CSV

## Setup

```sh
git clone https://github.com/mavmaso/ficherors
cd ficherors
cargo build --release
```

## Run

```sh
./target/release/ficherors
```

## Performance (Release mode)

Always build and run with `--release` for maximum performance. Debug builds skip all compiler optimisations and can be 10–50x slower.

For an even more aggressive optimisation, add this to `Cargo.toml`:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

Then:

```sh
cargo build --release
./target/release/ficherors
```

> `lto = true` enables Link-Time Optimisation across all crates.  
> `codegen-units = 1` forces a single compilation unit, giving the compiler maximum visibility for inlining and dead-code elimination.  
> Build time will be slower, but the binary will be as fast as possible.

## Benchmark

Build with `--release` and measure execution time with `time`:

```sh
cargo build --release
time ./target/release/ficherors
```

Example output (20 MB file on Apple Silicon, no country code, `has_accent: true`):

```
________________________________________________________
Executed in    1.06 secs      fish           external
   usr time  432.54 millis  154.00 micros  432.39 millis
   sys time   42.28 millis  575.00 micros   41.70 millis
```

> The bottleneck is CPU (usr time), not I/O (sys time).  
> Equivalent configuration to the Elixir and Zig benchmarks: empty `country_code`, `has_accent: true` (accent removal disabled).

With `country_code: "BR"` and accent removal enabled the same file takes ~1.57s.

## Test

```sh
cargo test
```

## Generating test files

The `escreve` script generates large sample CSV files for local testing (requires Elixir):

```sh
chmod +x ./escreve && ./escreve
```

It creates a `1giga.csv` with 28 million rows:

```
destination,name,org,correlation_id
11977775555,Daniel,Sinch,HD124 | 6
...
```

## Supported Countries

| Code | Country        | Code | Country        |
|------|----------------|------|----------------|
| BR   | Brazil         | MX   | Mexico         |
| AR   | Argentina      | PA   | Panama         |
| BO   | Bolivia        | PE   | Peru           |
| CL   | Chile          | PY   | Paraguay       |
| CO   | Colombia       | SV   | El Salvador    |
| CR   | Costa Rica     | UK   | United Kingdom |
| DO   | Dominican Rep. | US   | United States  |
| EC   | Ecuador        | UY   | Uruguay        |
| GT   | Guatemala      | VE   | Venezuela      |
| HN   | Honduras       | NI   | Nicaragua      |

## Column Functions

| Function     | Description                             |
|--------------|-----------------------------------------|
| `send_date`  | Today's date (`d/mm/yyyy`)              |
| `send_hour`  | Current time in a given timezone offset |
| `random_num` | Random integer 0–999                    |
| `downcase`   | Lowercase the value                     |
| `upcase`     | Uppercase the value                     |
| `first_word` | First space-separated word              |
| `first_down` | First word, lowercased                  |
| `fixed`      | Literal value from `target`             |
| `dynamic`    | Pass-through the source column value    |

## Made by

- [mavmaso](https://github.com/mavmaso)
