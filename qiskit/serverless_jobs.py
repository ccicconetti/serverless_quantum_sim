# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

import os

from qiskit_serverless import IBMServerlessClient
from ibm_credentials import get_ibm_credentials

ibm_credentials = get_ibm_credentials()

LOGS = bool(os.environ.get("LOGS", ""))
JOB = os.environ.get("JOB", "")

serverless = IBMServerlessClient(token=ibm_credentials["TOKEN"])

if JOB == "":
    for job in serverless.get_jobs():
        print("job_id {}, status {}".format(job.job_id, job.status()))
else:
    job = serverless.get_job_by_id(JOB)
    if LOGS:
        print(job.logs())
    else:
        if job.in_terminal_state():
            print(job.result(wait=False))
