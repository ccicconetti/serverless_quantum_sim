# Creation of a dataset of Qiskit serverless

Qiskit serverless can be installed in a local infrastructure by following
the [official instructions](https://qiskit.github.io/qiskit-serverless/).

The [HamLib dataset](https://portal.nersc.gov/cfs/m888/dcamps/hamlib/) is
used, in particular the max-cut problems with random 3-regular graphs.

## How to run

Download the HamLib dataset:


```shell
./download-dataset.sh
```

Create a Python virtual environment and activate it:

```shell
python3 -m virtualenv .venv
source .venv/bin/activate
```

Install the dependencies:

```shell
pip install -r requirements.txt
```

Execute the experiments:

```shell
python client.py
```

The dataset with experiments run at CNR-IIT can be downloaded with:

```shell
../scripts/download-artifacts.sh
```

To print basic stats:

```shell
python stats_single.py
python stats_series.py
```

## Dataset content

The dataset consists of two files, both in CSV format:

- `output_single.csv`: one row per experiment, with the following columns:
  - `dataset`: name of the dataset
  - `n_qubits`: number of qubits
  - `timestamp`: timestamp of when the experiment has been executed
  - `optimized_total`: total optimization time, in s
  - `num_iterations`: number of iterations run
  - `run_init`: time required to initialize the experiment, in s
  - `run_transpile`: time required to transpile the quantum circuit, in s
  - `run_vqe`: average time required for gradient descent, in s
  - `run_qc`: average time required for the execution of quantum circuit, in s
  - `run_sampler`: time required for post-processing, in s
  - `QUEUED`: time the job has remaining in a `QUEUED` state, in s
  - `INITIALIZING`: time the job has remaining in a `INITIALIZING` state, in s
  - `RUNNING`: time the job has remaining in a `RUNNING` state, in s
  - `avg_clas_iter_dur`: average time of a classical computing iteration, in s
- `output_series.csv`: multiple rows per experiment, with the following columns:
  - `dataset`: name of the dataset
  - `n_qubits`: number of qubits
  - `timestamp`: timestamp of when the experiment has been executed
  - `time`: iteration time (classical computing), in s
  - `cost`: iteration time (quantum computing), in s

These two files can be converted into datasets that can be loaded by the
simulator via the [a script provided](convert_output.py).