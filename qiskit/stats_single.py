#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt

filename = os.environ.get("DATASET", "output_single.csv")
pd.set_option("display.show_dimensions", False)
pd.set_option("display.max_columns", None)
pd.set_option("display.max_rows", None)
pd.set_option("display.max_colwidth", None)
df = pd.read_csv(filename, index_col=False)
df = df.select_dtypes(include="number")
print(df.describe())

columns = ["QUEUED", "INITIALIZING", "RUNNING", "num_iterations"]
fig, ax = plt.subplots(1, len(columns), figsize=(10, 5))
for i, column in zip(range(len(columns)), columns):
    if column == "num_iterations":
        ax[i].set_ylabel("Number of iterations")
    else:
        ax[i].set_ylabel("Time (s)")
    df.boxplot(column=[column], by=["n_qubits"], ax=ax[i])
plt.show()
