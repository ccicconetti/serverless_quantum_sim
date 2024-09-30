# 002-ibm-initial

## Set up

Execution times from traces obtained with a local Qiskit serverless deployment,
where the quantum computing times are normalized based on the estimated
execution on real IBM quantum computers.

Workload of VQE jobs with a mixed number of qubits (4, 6, 8, 10) and priorities
randomly assigned to 1, 2, or 4.

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