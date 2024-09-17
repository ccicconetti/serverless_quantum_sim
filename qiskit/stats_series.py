#!/usr/bin/env python3

import pandas as pd
import os

filename = os.environ.get("DATASET", "output_series.csv")
pd.set_option("display.show_dimensions", False)
df = pd.read_csv(filename)

metrics = ["exec_time", "cost_time"]
for metric in metrics:
    print("{}\n{}".format(metric, df.groupby(["n_qubits"])[metric].describe()))
