#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "output_series.csv")
SHOW = bool(os.environ.get("SHOW", ""))
N_QUBITS = os.environ.get("N_QUBITS", "")
LOG_SCALE = bool(os.environ.get("LOG_SCALE"))


def plot(
    df, x: str, y: str, ylabel: str | None, hue: str | None, show: bool, filename: str
):
    fig, ax = plt.subplots()
    sns.boxplot(df, x=x, y=y, hue=hue, ax=ax)
    ax.set_title("")
    ax.set_ylabel(ylabel)
    if LOG_SCALE:
        ax.set_yscale("log")
    fig.suptitle("")
    if show:
        plt.show(block=False)
    else:
        plt.savefig("{}.{}".format(filename, IMAGE_TYPE))


pd.set_option("display.show_dimensions", False)
df = pd.read_csv(DATASET)

n_qubits = list(filter(None,N_QUBITS.strip().split(",")))
n_qubits = [int(x) for x in n_qubits]
if n_qubits != []:
    df = df[df["n_qubits"].isin(n_qubits)]

metrics = [
    ("exec_time", "Quantum task execution time (ms)"),
    ("cost_time", "Classical task execution time (ms)"),
]
for ymetric, ylabel in metrics:
    df[ymetric] = df[ymetric] * 1000.0
    plot(
        df,
        x="n_qubits",
        y=ymetric,
        ylabel=ylabel,
        hue=None,
        show=SHOW,
        filename="{}-{}".format(os.path.basename(os.getcwd()), ymetric),
    )

if SHOW:
    input("Press any key to continue")
