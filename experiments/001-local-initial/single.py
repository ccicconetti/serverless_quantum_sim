#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "data/single.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(df, by: list, column: str, ylabel: str, show: bool, filename: str):
    fig, ax = plt.subplots(1, 1, figsize=(len(by) * 5, 5))
    df.boxplot(column=[column], by=by, ax=ax)
    ax.set_ylabel(ylabel)
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
df["drop_prob"] = df["num_job_dropped"] / (
    df["num_job_accepted"] + df["num_job_dropped"]
)
df["quantum_tasks"] = df["active_quantum_tasks"] + df["pending_quantum_tasks"]

metrics = [
    ("drop_prob", "Drop probability"),
    ("quantum_tasks", "Average quantum tasks"),
    ("active_classical_tasks", "Average classical tasks"),
]
for column, ylabel in metrics:
    plot(
        df,
        by=["policy"],
        column=column,
        ylabel=ylabel,
        show=SHOW,
        filename="001-single-{}".format(column),
    )

if SHOW:
    input("Press any key to continue")
