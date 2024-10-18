#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "ibm_job_estimate.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(
    df,
    x: str,
    xlabel: str | None,
    y: str,
    ylabel: str | None,
    hue: str | None,
    show: bool,
    filename: str,
):
    fig, ax = plt.subplots()
    sns.barplot(df, x=x, y=y, hue=hue, ax=ax)
    ax.set_title("")
    ax.set_xlabel(xlabel)
    ax.set_ylabel(ylabel)
    ax.set_ylim(top=7)
    ax.bar_label(
        ax.containers[0],
        labels=df["backend"].values,
        rotation=90,
        fontsize=7,
        padding=3,
    )
    fig.suptitle("")
    if show:
        plt.show(block=False)
    else:
        plt.savefig("{}.{}".format(filename, IMAGE_TYPE))


df = pd.read_csv(
    DATASET,
    index_col=False,
    names=["timestamp", "n_qubits", "backend", "value"],
)
df["backend"] = df["backend"].str.replace("ibm_", "")

plot(
    df,
    x="n_qubits",
    xlabel="n_qubits",
    y="value",
    ylabel="Estimated quantum circuit execution duration (s)",
    hue=None,
    show=SHOW,
    filename="{}-ibm_estimate".format(os.path.basename(os.getcwd())),
)


if SHOW:
    input("Press any key to continue")
