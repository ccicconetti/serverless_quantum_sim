#!/usr/bin/env python3

import pandas as pd
import os

filename = os.environ.get("DATASET", "output_single.csv")
pd.set_option("display.show_dimensions", False)
df = pd.read_csv(filename)

print(df.describe())
