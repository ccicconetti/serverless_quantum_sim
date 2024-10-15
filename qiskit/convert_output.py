#!/usr/bin/env python3

import pandas as pd
import os

filename_single = os.environ.get("DATASET_SINGLE", "output_single.csv")
df = pd.read_csv(filename_single, index_col=False)
df = df.select_dtypes(include="number")

df["pre"] = df["QUEUED"] + df["INITIALIZING"] + df["run_transpile"] + df["run_init"]
df.rename(
    columns={"avg_clas_iter_dur": "cost_time", "run_sampler": "post"}, inplace=True
)
print(df)

metrics = ["pre", "post", "num_iterations", "cost_time"]
for metric in metrics:
    df.to_csv("{}.csv".format(metric), columns=["n_qubits", metric], index=False)

filename_series = os.environ.get("DATASET_SERIES", "output_series.csv")
df = pd.read_csv(filename_series, index_col=False)
df = df.select_dtypes(include="number")
df.rename(columns={"time": "exec_time"}, inplace=True)

metrics = ["exec_time"]
for metric in metrics:
    df.to_csv("{}.csv".format(metric), columns=["n_qubits", metric], index=False)
