# 003-local-var-load

## Set up

Execution times from traces obtained with a local Qiskit serverless deployment.

Workload of VQE jobs with a mixed number of qubits (4, 6, 8, 10), all with
the same priority, with varying:

- interarrival time between consecutive jobs, and
- number of (classical) serverless workers.

## Reproducibility

To reproduce the experiment:

```shell
./run.sh
```

To create the plots make sure that the libraries `seaborn`, `matplotlib`, and
`pandas` are installed and then run the following scripts:

```shell
python single.py
python job_time.py
```