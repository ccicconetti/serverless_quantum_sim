#!/usr/bin/env python3

import pandas as pd
import os
import matplotlib.pyplot as plt
import seaborn as sns

IMAGE_TYPE = os.environ.get("IMAGE_TYPE", "png")
DATASET = os.environ.get("DATASET", "data/single.csv")
SHOW = bool(os.environ.get("SHOW", ""))


def plot(
    df, x: str, y: str, hue: str | None, hue_label: str, show: bool, filename: str
):
    fig, ax = plt.subplots()
    table = pd.pivot_table(df, index=x, columns=y, values=hue)
    sns.heatmap(
        table, cmap="coolwarm", annot=True, fmt="0.0f", cbar_kws={"label": hue_label}
    )
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
df = df.rename(columns={"job_interarrival": "interarrival"})
df["drop_prob"] = (
    100 * df["num_job_dropped"] / (df["num_job_accepted"] + df["num_job_dropped"])
)
df["quantum_tasks"] = df["active_quantum_tasks"] + df["pending_quantum_tasks"]
df["job_rate"] = 3600 * df["num_job_accepted"] / df["duration"]


metrics = [
    ("drop_prob", "Drop probability (%)"),
    ("quantum_tasks", "Average quantum tasks"),
    ("active_classical_tasks", "Average classical tasks"),
    ("job_rate", "Job rate (1/hour)"),
]
for hue, hue_label in metrics:
    plot(
        df,
        x="interarrival",
        y="C",
        hue_label=hue_label,
        hue=hue,
        show=SHOW,
        filename="{}-{}".format(os.path.basename(os.getcwd()), hue),
    )

if SHOW:
    input("Press any key to continue")
