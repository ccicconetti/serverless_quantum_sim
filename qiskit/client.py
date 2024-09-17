# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT
#
# Inspired from the example code provided by Qiskit serverless for VQE.
# The original code is available on GitHub:
# https://github.com/Qiskit/qiskit-serverless


import os
import time
import logging
import datetime
from qiskit_serverless import QiskitFunction, ServerlessClient
from prepare_input import prepare_input, get_datasets

# Files where to save the output of the experiments
OUTPUT_SINGLE = "output_single.csv"
OUTPUT_SERIES = "output_series.csv"

# Logging configuration
logging.basicConfig(
    format="%(asctime)s.%(msecs)03d %(levelname)-8s %(message)s",
    level=logging.INFO,
    datefmt="%Y-%m-%d %H:%M:%S",
)

# Create a client that connects to a local cluster
serverless = ServerlessClient(
    token=os.environ.get("GATEWAY_TOKEN", "awesome_token"),
    host=os.environ.get("GATEWAY_HOST", "http://localhost:8000"),
)

# Create and upload the VQE function
function = QiskitFunction(
    title="vqe",
    entrypoint="vqe.py",
    working_dir="function",
    dependencies=["qiskit_aer"],
)
serverless.upload(function)

# Get all the datasets
datasets = get_datasets(max_qubits=4)

# Run the experiment
for dataset in datasets:
    # Get a timestamp.
    timestamp = int(time.time())

    input_arguments = prepare_input(dataset)

    logging.info("starting with input arguments: {}".format(input_arguments))

    job = serverless.run("vqe", arguments=input_arguments)
    logging.info("job ID: {}".format(job.job_id))

    timestamps = {}
    states = []

    while True:
        status = job.status()
        if status not in timestamps:
            timestamps[status] = datetime.datetime.now()
            states.append(status)
        if status == "DONE":
            break
        elif status == "ERROR":
            raise RuntimeError("the job could not be run")
        time.sleep(0.01)

    for cur, next in zip(range(len(states) - 1), range(1, len(states))):
        s_cur = states[cur]
        s_next = states[next]
        assert timestamps[s_next] >= timestamps[s_cur]
        delta = timestamps[s_next] - timestamps[s_cur]
        logging.info("state {} duration: {} s".format(s_cur, delta.total_seconds()))

    logging.info("finished: {}".format(job.result()))
