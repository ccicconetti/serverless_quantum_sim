# serverless_quantum_sim

The repository contains two contributions:

1. A simulator of a hybrid quantum-classical infrastructure running jobs
following a serverless approach. The instructions to
build and execute the simulator can be found below. The simulator uses as input
some datasets that have been produced by executing serverless experiments with
Qiskit serverless on the computing infrastructure of the 
[Ubiquitous Internet group](https://ui.iit.cnr.it/en/) of
[IIT-CNR](https://www.iit.cnr.it/en/claudio.cicconetti/).
2. The Python scripts that have been used to produce the datasets needed as
input by the simulator. Please refer to the
[specific documentation](qiskit/README.md) for more information on
this contribution.

## Serverless quantum-classical simulator

The simulator is written in the
[Rust programming language](https://www.rust-lang.org/). 


### Building

Install Rust by following the interactive instructions from here:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Clone the repo (if not already done):

```shell
git clone https://github.com/ccicconetti/serverless_quantum_sim.git
cd serverless_quantum_sim
```

Build the system in debug mode:

```shell
cargo build
```

This will build an executable file `target/debug/serverless_quantum_sim`  
with no optimizations that can be used to try or debug the software.

[_Optional_] Run the unit tests:

```shell
cargo test
```

[_Optional_] Build the system in release mode to run heavy experiments
(the executable file is `target/release/serverless_quantum_sim`):

```shell
cargo build --release
```

### Input datasets

The simulator assumes that the current working directory contains 
a directory `input/` with the following input datasets:

| Name                 | Description                                                         |
| -------------------- | ------------------------------------------------------------------- |
| `cost_time.csv`      | Time required by classical computing for a single iteration         |
| `exec_time.csv`      | Time required by quantum computing for a single iteration           |
| `num_iterations.csv` | Number of iterations                                                |
| `post.csv`           | Time required by classical computing for post-processing operations |
| `pre.csv`            | Time required by classical computing for initial operations         |

Each dataset is a CSV file containing two columns: the problem size (in 
number of qubits) and the value of interest (see table above).
All times are in seconds.

The datasets are used by the simulator to configure the empiric random variables
that drive the system dynamics.

A collection of input datasets are provided with the repo.
They have been generated with the tools described [here](qiskit/README.md).

### Execution

You can see the command-line options with:

```shell
target/debug/serverless_quantum_sim --help
```

When executed, the simulator runs a number of replications in parallel (the
maximum number of thread to use can be set via a command-line option) and it
produces two output files.

For instance, when run with no options:

```shell
target/debug/serverless_quantum_sim
```

It will produce two files:

- `data/single.csv`: A CSV file containing one row for each replication. The first columns save the configuration of the experiment, while the others are the simulation output. There is a header that explains the meaning of each column.
- `data/job_time.csv`: A CSV file containing, for each replication, the durations of all the jobs completed (in seconds) in the last column.

There are some complete experiments in `experiments`, each with Bash scripts to run the simulations and with Python scripts to visualize relevant results, which can be easily adapted to run further experiments.