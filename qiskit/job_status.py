# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

import os

from qiskit_ibm_runtime import QiskitRuntimeService
from ibm_credentials import get_ibm_credentials

ibm_credentials = get_ibm_credentials()

JOB = os.environ.get("JOB", "")
assert JOB != ""

service = QiskitRuntimeService(
    channel=ibm_credentials["CHANNEL"],
    instance=ibm_credentials["INSTANCE"],
    token=ibm_credentials["TOKEN"],
)

try:
    job = service.job(JOB)
    status = job.status()
    if status == "DONE":
        for idx, pub_result in enumerate(job.result()):
            print(f"Expectation values for pub {idx}: {pub_result.data.evs}")

    elif status == "ERROR":
        print(job.logs())
    else:
        print(status)
except Exception as err:
    print("error: {}".format(err))
