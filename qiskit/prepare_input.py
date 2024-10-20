# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

from qiskit.circuit.library import EfficientSU2

from hamlib_read import read_openfermion_hdf5, get_hdf5_keys
from qubitop_to_pauliop import qubitop_to_pauliop
from qiskit_ibm_runtime import QiskitRuntimeService

HAM_FILENAME = "ham-graph-regular_reg-3.hdf5"
LABELS = ["reg", "n", "rinst"]


def dataset_name(values: list):
    fields = []
    for i in range(len(values)):
        value = values[i]
        if i == 2:
            value = str(value).rjust(2, "0")
        fields.append(f"{LABELS[i]}-{value}")
    return "{}".format("_".join(fields))


def get_datasets(min_qubits: int, max_qubits: int):
    keys = get_hdf5_keys(HAM_FILENAME)
    datasets = []

    for key in keys:
        tokens = key.replace(",", "").replace("/", "").split("_")
        assert len(tokens) == len(LABELS)
        entry = []
        for i in range(len(LABELS)):
            subtokens = tokens[i].split("-")
            assert len(subtokens) == 2
            assert subtokens[0] == LABELS[i]
            value = int(subtokens[1])
            if i == 1 and (value < min_qubits or value > max_qubits):
                break
            entry.append(value)
        if len(entry) == len(LABELS):
            datasets.append(entry)
    assert datasets != []

    return datasets


def prepare_input(dataset: str, maxiter: int, ibm_credentials: dict | None):

    service = None
    if ibm_credentials is not None:
        assert "CHANNEL" in ibm_credentials and ibm_credentials["CHANNEL"] != ""
        assert "INSTANCE" in ibm_credentials and ibm_credentials["INSTANCE"] != ""
        assert "TOKEN" in ibm_credentials and ibm_credentials["TOKEN"] != ""
        service = QiskitRuntimeService(
            channel=ibm_credentials["CHANNEL"],
            instance=ibm_credentials["INSTANCE"],
            token=ibm_credentials["TOKEN"],
            verify=False,
        )

    of = read_openfermion_hdf5(HAM_FILENAME, key=dataset)
    operator = qubitop_to_pauliop(of)

    ansatz = EfficientSU2(operator.num_qubits)

    input_arguments = {
        "ansatz": ansatz,
        "operator": operator,
        "method": "COBYLA",
        "n_qubits": operator.num_qubits,
        "dataset": dataset,
        "service": service,
        "maxiter": maxiter,
    }

    return input_arguments
