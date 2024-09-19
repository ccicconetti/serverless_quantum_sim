#!/usr/bin/env python3

import pandas as pd
import os

filename_single = os.environ.get("DATASET_SINGLE", "output_single.csv")
df = pd.read_csv(filename_single, index_col=False)
df = df.select_dtypes(include="number")

df["pre"] = df["QUEUED"] + df["INITIALIZING"]

metrics = ["pre", "run_sampler", "num_iterations"]
for metric in metrics:
    df.to_csv("{}.csv".format(metric), columns=["n_qubits", metric], index=False)

filename_series = os.environ.get("DATASET_SERIES", "output_series.csv")
df = pd.read_csv(filename_series, index_col=False)
df = df.select_dtypes(include="number")

metrics = ["cost_time", "exec_time"]
for metric in metrics:
    df.to_csv("{}.csv".format(metric), columns=["n_qubits", metric], index=False)
