## How to run

You can run with:

```
cargo run -- input.csv > output.csv
```

The first parameter is the path to csv input file.

## Files

Here are the key files:

1. `types.rs` contains types used in this project, including `AccountProfile`, `Transaction` and more.
2. `transaction.rs` contains the core logic to process transaction.
3. `main.rs` handles CSV IO and integration.

## Testing

The `transaction.rs` file contains a few unit tests for the core logic of transaction processing.

I also tested it end to end with an example CSV input. (I didn't commit those files as instructed)

## Notes and Assumptions

1. We ignore all errors silently (instead of output to stderr) for input parsing and transaction rejection.
2. We will output the clients in arbitrary order
3. We assume you can only dispute a "deposit" and no other type of transactions.
4. We assume txn_id should be unique among all deposit and withdrawal within one client, we will reject duplications.
5. When we dispute a transaction, if it will result in a negative available balance (user already withdrawal), we will
   reject it.
6. We read input CSV file incrementally.
7. Due to the serial nature of a CSV file we didn't introduce concurrency in the code.

## AI tools usage

1. I used ChatGPT to generate snippet about CSV reading and parsing.
   Prompt used:

```
In Rust, how to parse a csv file line by line so I don't need to store the whole thing in memory?
I want to parse it row by row into a specific struct.
```

2. I used ChatGPT to generate snippet about command line argument parsing. Prompt used:

```
In Rust, how to parse command line args?
```

3. I use RustRover as my IDE and occasionally rely on its built-in "local Full Line completion suggestions" features
   during development.
