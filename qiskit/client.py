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
from prepare_input import prepare_input, get_datasets, dataset_name

# Files where to save the output of the experiments
OUTPUT_SINGLE = "output_single.csv"
OUTPUT_SERIES = "output_series.csv"


def dump_data(
    input_arguments: dict, timestamp: int, results: dict, status_durations: dict
):
    header = ""
    if not os.path.exists(OUTPUT_SINGLE) or os.path.getsize(OUTPUT_SINGLE) == 0:
        header = "dataset,n_qubits,timestamp,optimized_total,num_iterations,"
        header += ",".join(results["durations"].keys())
        header += ","
        header += ",".join(status_durations.keys())
        header += "\n"
    with open(OUTPUT_SINGLE, "a") as outfile:
        if header != "":
            outfile.write(header)
        outfile.write(
            "{},{},{},{},{},{},{}\n".format(
                input_arguments["dataset"],
                input_arguments["n_qubits"],
                timestamp,
                results["optimizer_time"],
                results["num_iterations"],
                ",".join(str(x) for x in results["durations"].values()),
                ",".join(str(x) for x in status_durations.values()),
            )
        )

    header = ""
    if not os.path.exists(OUTPUT_SERIES) or os.path.getsize(OUTPUT_SERIES) == 0:
        header = "dataset,n_qubits,timestamp,exec_time,cost_time\n"
    assert len(results["exec_times"]) == len(results["cost_times"])
    with open(OUTPUT_SERIES, "a") as outfile:
        if header != "":
            outfile.write(header)
        for exec_time, cost_time in zip(results["exec_times"], results["cost_times"]):
            outfile.write(
                "{},{},{},{},{}\n".format(
                    input_arguments["dataset"],
                    input_arguments["n_qubits"],
                    timestamp,
                    exec_time,
                    cost_time,
                )
            )


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
datasets = get_datasets(max_qubits=10)

# Run the experiment
for dataset in datasets:
    # Get a timestamp.
    timestamp = int(time.time())

    # Prepare the function arguments
    input_arguments = prepare_input(dataset_name(dataset))

    logging.info("starting with input arguments: {}".format(input_arguments))

    # Dispatch the job to the serverless platform
    job = serverless.run("vqe", arguments=input_arguments)
    logging.info("job ID: {}".format(job.job_id))

    timestamps = {}
    states = []

    while True:
        status = job.status()
        if status not in timestamps:
            logging.info(status)
            timestamps[status] = datetime.datetime.now()
            states.append(status)
        if status == "DONE":
            break
        elif status == "ERROR":
            raise RuntimeError("the job could not be run")
        time.sleep(0.01)

    logging.info("dumping data")

    # Compute duration of each status
    status_durations = {}
    for cur, next in zip(range(len(states) - 1), range(1, len(states))):
        s_cur = states[cur]
        s_next = states[next]
        assert timestamps[s_next] >= timestamps[s_cur]
        delta = timestamps[s_next] - timestamps[s_cur]
        status_durations[s_cur] = delta.total_seconds()

    # Dump data to the output file
    dump_data(input_arguments, timestamp, job.result(), status_durations)
