#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "data/job_time.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(df, by: list, show: bool, filename: str):
    fig, ax = plt.subplots(1, 1, figsize=(len(by) * 5, 5))
    df.boxplot(column=["value"], by=by, ax=ax)
    ax.set_ylabel("Job execution (s)")
    ax.set_title("")
    fig.suptitle("")
    if show:
        plt.show(block=False)
    else:
        plt.savefig("{}.{}".format(filename, IMAGE_TYPE))


pd.set_option("display.show_dimensions", False)
pd.set_option("display.max_columns", None)
pd.set_option("display.max_colwidth", None)
df = pd.read_csv(DATASET)
df = df.rename(columns={"quantum_schedule_policy": "policy"})
df = df.replace({"fifo": "F", "lifo": "L", "random": "R", "weighted": "W"})

plot(
    df,
    by=["policy", "num_qubits"],
    show=SHOW,
    filename="001-job_time-box-num_qubits",
)
plot(
    df,
    by=["policy", "priority"],
    show=SHOW,
    filename="001-job_time-box-priority",
)
plot(
    df,
    by=["policy"],
    show=SHOW,
    filename="001-job_time-box",
)

if SHOW:
    input("Press any key to continue")
