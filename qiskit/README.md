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