#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "output_single.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(
    df, x: str, y: str, ylabel: str | None, hue: str | None, show: bool, filename: str
):
    fig, ax = plt.subplots()
    sns.boxplot(df, x=x, y=y, hue=hue, ax=ax)
    ax.set_title("")
    ax.set_ylabel(ylabel)
    fig.suptitle("")
    if show:
        plt.show(block=False)
    else:
        plt.savefig("{}.{}".format(filename, IMAGE_TYPE))


pd.set_option("display.show_dimensions", False)
pd.set_option("display.max_columns", None)
pd.set_option("display.max_rows", None)
pd.set_option("display.max_colwidth", None)
df = pd.read_csv(DATASET, index_col=False)
df = df.select_dtypes(include="number")
print(df.describe())

metrics = [
    ("QUEUED", "Time (s)"),
    ("INITIALIZING", "Time (s)"),
    ("RUNNING", "Time (s)"),
    ("num_iterations", "Number of iterations"),
]

for ymetric, ylabel in metrics:
    plot(
        df,
        x="n_qubits",
        y=ymetric,
        ylabel=ymetric,
        hue=None,
        show=SHOW,
        filename="{}-{}".format(os.path.basename(os.getcwd()), ymetric),
    )

if SHOW:
    input("Press any key to continue")
