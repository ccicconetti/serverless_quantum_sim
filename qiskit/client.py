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
from qiskit_serverless import QiskitFunction, ServerlessClient, IBMServerlessClient
from prepare_input import prepare_input, get_datasets, dataset_name
from ibm_credentials import get_ibm_credentials

# Files where to save the output of the experiments
OUTPUT_SINGLE = "output_single.csv"
OUTPUT_SERIES = "output_series.csv"

# Options
DRY = bool(os.environ.get("DRY", ""))
MIN_QUBITS = int(os.environ.get("MIN_QUBITS", "4"))
MAX_QUBITS = int(os.environ.get("MAX_QUBITS", "10"))
NUM_RUNS = os.environ.get("NUM_RUNS", "")
MAX_ITER = int(os.environ.get("MAX_ITER", "1000"))


def dump_data(
    input_arguments: dict, timestamp: int, results: dict, status_durations: dict
):
    assert len(results["times"]) == len(results["costs"])

    # sometimes (but not always) there is one last spurious cost_func invocation
    if len(results["times"]) != results["num_iterations"]:
        logging.warning(
            "num_iterations = {}, cost_func invocations = {}".format(
                results["num_iterations"], len(results["times"])
            )
        )
        results["num_iterations"] = len(results["times"])

    header = ""
    if not os.path.exists(OUTPUT_SINGLE) or os.path.getsize(OUTPUT_SINGLE) == 0:
        header = "dataset,n_qubits,timestamp,optimized_total,num_iterations,"
        header += ",".join(results["durations"].keys())
        header += ","
        header += ",".join(status_durations.keys())
        header += ",avg_clas_iter_dur"
        header += "\n"

    avg_clas_iter_dur = (
        float(results["durations"]["run_vqe"]) - sum(results["times"])
    ) / (float(results["num_iterations"]))
    with open(OUTPUT_SINGLE, "a") as outfile:
        if header != "":
            outfile.write(header)
        outfile.write(
            "{},{},{},{},{},{},{},{}\n".format(
                input_arguments["dataset"],
                input_arguments["n_qubits"],
                timestamp,
                results["optimizer_time"],
                results["num_iterations"],
                ",".join(str(x) for x in results["durations"].values()),
                ",".join(str(x) for x in status_durations.values()),
                avg_clas_iter_dur,
            )
        )

    header = ""
    if not os.path.exists(OUTPUT_SERIES) or os.path.getsize(OUTPUT_SERIES) == 0:
        header = "dataset,n_qubits,timestamp,time,cost\n"

    with open(OUTPUT_SERIES, "a") as outfile:
        if header != "":
            outfile.write(header)
        for time, cost in zip(results["times"], results["costs"]):
            outfile.write(
                "{},{},{},{},{}\n".format(
                    input_arguments["dataset"],
                    input_arguments["n_qubits"],
                    timestamp,
                    time,
                    cost,
                )
            )


# Logging configuration
logging.basicConfig(
    format="%(asctime)s.%(msecs)03d %(levelname)-8s %(message)s",
    level=logging.INFO,
    datefmt="%Y-%m-%d %H:%M:%S",
)

# Check IBM credentials
ibm_credentials = get_ibm_credentials()
logging.info("IBM credentials: {}".format(ibm_credentials))

# Log options
logging.info("Options:")
logging.info(f"OUTPUT_SINGLE = {OUTPUT_SINGLE}")
logging.info(f"OUTPUT_SERIES = {OUTPUT_SERIES}")
logging.info(f"DRY = {DRY}")
logging.info(f"MIN_QUBITS = {MIN_QUBITS}")
logging.info(f"MAX_QUBITS = {MAX_QUBITS}")
logging.info(f"NUM_RUNS = {NUM_RUNS}")
logging.info(f"MAX_ITER = {MAX_ITER}")

# Get all the datasets
datasets = get_datasets(min_qubits=MIN_QUBITS, max_qubits=MAX_QUBITS)

if NUM_RUNS != "":
    datasets = datasets[0 : int(NUM_RUNS)]

if DRY:
    for dataset in datasets:
        print(dataset)
    quit()

# Create a client that connects to a local cluster
if ibm_credentials is None:
    serverless = ServerlessClient(
        token=os.environ.get("GATEWAY_TOKEN", "awesome_token"),
        host=os.environ.get("GATEWAY_HOST", "http://localhost:8000"),
    )
else:
    print(
        "You are about to start an experiment on a real IBM backend, enter 'yes' to confirm"
    )
    res = input()
    if res.lower() != "yes":
        os._exit()
    serverless = IBMServerlessClient(token=ibm_credentials["TOKEN"])

# Create and upload the VQE function
dependencies = []
if ibm_credentials is None:
    dependencies = ["qiskit_aer"]
function = QiskitFunction(
    title="vqe",
    entrypoint="vqe.py",
    working_dir="function",
    dependencies=dependencies,
)
serverless.upload(function)

# Run the experiment
for dataset in datasets:
    # Get a timestamp.
    timestamp = int(time.time())

    # Prepare the function arguments
    input_arguments = prepare_input(
        dataset_name(dataset), maxiter=MAX_ITER, ibm_credentials=ibm_credentials
    )

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
            logging.error(job.logs())
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
