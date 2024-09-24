# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

import os


def get_ibm_credentials() -> dict:
    CHANNEL = os.environ.get("IBM_CHANNEL", "")
    INSTANCE = os.environ.get("IBM_INSTANCE", "")
    TOKEN = os.environ.get("IBM_TOKEN", "")

    ibm_credentials = None
    if CHANNEL == "ibm_quantum" and TOKEN != "" and INSTANCE != "":
        ibm_credentials = {}
        ibm_credentials["CHANNEL"] = CHANNEL
        ibm_credentials["INSTANCE"] = INSTANCE
        ibm_credentials["TOKEN"] = TOKEN

    return ibm_credentials
