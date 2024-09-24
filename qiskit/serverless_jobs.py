# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

from qiskit_serverless import IBMServerlessClient
from ibm_credentials import get_ibm_credentials

ibm_credentials = get_ibm_credentials()

serverless = IBMServerlessClient(token=ibm_credentials["TOKEN"])
for job in serverless.get_jobs():
    print("job_id {}, status {}".format(job.job_id, job.status()))
