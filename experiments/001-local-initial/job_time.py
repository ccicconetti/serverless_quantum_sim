#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "data/job_time.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(df, x: str, hue: str | None, show: bool, filename: str):
    fig, ax = plt.subplots()
    sns.boxplot(df, y="value", x=x, hue=hue, ax=ax)
    ax.set_ylabel("Job execution (s)")
    ax.set_title("")
    ax.set_yscale("log")
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

basename = os.path.basename(os.getcwd())

plot(
    df,
    x="policy",
    hue="num_qubits",
    show=SHOW,
    filename="{}-job_time-box-num_qubits".format(basename),
)
plot(
    df,
    x="policy",
    hue="priority",
    show=SHOW,
    filename="{}-job_time-box-priority".format(basename),
)
plot(
    df,
    x="policy",
    hue=None,
    show=SHOW,
    filename="{}-job_time-box".format(basename),
)

if SHOW:
    input("Press any key to continue")
