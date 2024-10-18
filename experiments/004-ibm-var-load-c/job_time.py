#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "data/job_time.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(df, x: str, xlabel: str, hue: str | None, show: bool, filename: str):
    fig, ax = plt.subplots()
    sns.boxplot(df, y="value", x=x, hue=hue, ax=ax)
    ax.set_ylabel("Job execution (hours)")
    ax.set_xlabel(xlabel)
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
df = df.rename(columns={"num_serverless_workers": "C"})
df = df.rename(columns={"num_quantum_computers": "Q"})
df = df.rename(columns={"job_interarrival": "interarrival"})
df["value"] = df["value"] / 3600
df["load"] = 3600 / df["interarrival"]

basename = os.path.basename(os.getcwd())

plot(
    df,
    x="load",
    xlabel="Load (jobs/hour)",
    hue="C",
    show=SHOW,
    filename="{}-job_time-all".format(basename),
)

for interarrival in df["interarrival"].unique():
    plot(
        df[df["interarrival"] == interarrival],
        x="C",
        xlabel="C",
        hue=None,
        show=SHOW,
        filename="{}-job_time-{}".format(basename, interarrival),
    )


if SHOW:
    input("Press any key to continue")
