# Benchmark individual proofs.

In this section we will cover how to run the benchmarks for the individual proofs. The benchmarks are located in
the `light-client` crate folder.

Benchmarks can be classified in two distinct categories:

- _end-to-end_: Those benchmarks are associated with programs that are meant to reproduce
  a production environment settings. They are meant to measure performance for a complete
  end-to-end flow.
- _internals_: Those benchmarks are associated with programs that are solely meant for
  performance measurements on specific parts of the codebase. They are
  not meant to measure performance for, or reproduce a production environment settings.

## End-to-end

- [e2e](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/light-client/benches/e2e.rs): Benchmark that will run a proof generation for both the [epoch change program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/programs/epoch-change/src/main.rs)
  and the [inclusion program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/programs/inclusion/src/main.rs).
  The goal here is to test the complete flow for our light client and get cycle count and proving time for it. Note that
  each proof is handled sequentially, so running it might take some time.
- [epoch_change](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/light-client/benches/epoch_change.rs): Benchmark that will run a proof generation
  for [epoch change program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/programs/epoch-change/src/main.rs).
  This program will execute a hash for the received `ValidatorVerifier` to ensure that the signature is from the
  previous validator set, execute a `TrustedState::verify_and_ratchet_inner` and finally generate the hash for the
  verified `ValidatorVerifier`.
- [inclusion](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/light-client/benches/inclusion.rs): Benchmark that will run a proof generation for the [inclusion program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/programs/inclusion/src/main.rs).
  It is meant to assess the cost of verifying a Merkle proof for a given leaf and a given root.

## Internals

- [sig](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/light-client/benches/sig.rs): Benchmark that will run a proof generation for the [signature verification program](https://github.com/argumentcomputer/zk-light-clients/blob/dev/aptos/programs/benchmarks/signature-verification/src/main.rs).
  This program mainly executes an aggregated signature verification for an aggregated signature and a set of public
  keys.

The benchmark that is the closest to a production scenario is `e2e`. Most of
the other benchmarks are more specific and are meant to assess the cost
of specific operations.

### Running the benchmarks

**Using Makefile**

To ease benchmark run we created a Makefile in the `light-client` crate folder.
Just run:

```shell
make benchmark
```

You will then be asked for the name of the benchmark you want to run. Just
fill in the one that is of interest to you:

```shell
$ make benchmark
Enter benchmark name: e2e

  ...
  
```

> **Info**
>
> For the `epoch_change`, `inclusion`, and `sig` benchmarks, you can measure the
> time to generate a SNARK proof by passing the `MODE="SNARK"` environment variable.

**Manual**

Run the following command:

```shell
SHARD_BATCH_SIZE=0 cargo bench --features aptos --bench execute -- <benchmark_name>
```

## Interpreting the results

Before delving into the details, please take a look at the [cycle tracking documentation
from SP1](https://succinctlabs.github.io/sp1/writing-programs/cycle-tracking.html) to get a rough sense of what the
numbers mean.

The benchmark will output a lot of information. The most important parts are the
following:

**Total cycles for the program execution**

This value can be found on the following line:

```shell
INFO summary: cycles=63736, e2e=2506, khz=25.43, proofSize=2.66 MiB
```

It contains the total number of cycles needed for the program, the end-to-end time in milliseconds, the frequency of the
CPU in kHz, and the size of the proof generated.

**Specific cycle count**

In the output, you will find a section that looks like this:

```shell
DEBUG ┌╴read_inputs    
DEBUG └╴9,553 cycles    
DEBUG ┌╴verify_merkle_proof    
DEBUG └╴40,398 cycles    
```

These specific cycles count are generated by us to track the cost of specific operations in the program.

**Proving time**

The proving time is an output at the end of a benchmark in the shape of the following data structure, with each time in
milliseconds:

```json
{
  ratchet_proving_time: 100000,
  merkle_proving_time: 100000
}
```

## Alternative

Another solution to get some information about proving time is to run the tests located in the `light-client`
crate. They will output the same logs as the benchmarks, only the time necessary
to generate a proof will change shape:

```shell
Starting generation of Merkle inclusion proof with 18 siblings...
Proving locally
Proving took 5.358508094s
Starting verification of Merkle inclusion proof...
Verification took 805.530068ms
```

To run the test efficiently, first install `nextest` following [its documentation](https://nexte.st/book/installation).
Ensure that you also have the previously described environment variables set, then run the following command:

```shell
SHARD_BATCH_SIZE=0 cargo nextest run --verbose --release --profile ci --features aptos --package aptos-lc --no-capture
```

> **Note**
>
> The `--no-capture` flag is necessary to see the logs generated by the tests.

Some tests are ignored by default due to heavier resource requirements. To run them, pass `--run-ignored all`
to `nextest`.

A short list of useful tests:

- `test_execute_epoch_change`: Executes the `epoch_change` program inside the zkVM but does not generate any proofs.
- `test_prove_epoch_change`: Generates and verifies a STARK proof of the `epoch_change` program.
- `test_snark_prove_epoch_change`: Generates and verifies a SNARK proof of the `epoch_change` program.
- `test_execute_inclusion`: Executes the `epoch_change` program inside the zkVM but does not generate any proofs.
- `test_prove_inclusion`: Generates and verifies a STARK proof of the `epoch_change` program.
- `test_snark_inclusion`: Generates and verifies a SNARK proof of the `epoch_change` program.
