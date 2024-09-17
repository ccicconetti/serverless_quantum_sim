# SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
# SPDX-License-Identifier: MIT

from qiskit.circuit.library import EfficientSU2
from random import choice

from hamlib_read import read_openfermion_hdf5, get_hdf5_keys
from qubitop_to_pauliop import qubitop_to_pauliop

HAM_FILENAME = "ham-graph-regular_reg-3.hdf5"
LABELS = ["reg", "n", "rinst"]


def random_dataset(datasets: list):
    values = choice(datasets)
    assert len(values) == len(LABELS)
    fields = []
    for i in range(len(values)):
        value = values[i]
        if i == 2:
            value = str(value).rjust(2, "0")
        fields.append(f"{LABELS[i]}-{value}")
    return "{}".format("_".join(fields))


def get_datasets(max_qubits: int):
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
            if i == 1 and value > max_qubits:
                break
            entry.append(value)
        if len(entry) == len(LABELS):
            datasets.append(entry)
    assert datasets != []

    return datasets


def prepare_input(max_qubits: int):

    datasets = get_datasets(max_qubits)
    dataset_name = random_dataset(datasets=datasets)
    of = read_openfermion_hdf5(HAM_FILENAME, key=dataset_name)
    operator = qubitop_to_pauliop(of)

    ansatz = EfficientSU2(operator.num_qubits)

    input_arguments = {
        "ansatz": ansatz,
        "operator": operator,
        "method": "COBYLA",
        "n_qubits": operator.num_qubits,
        "dataset": dataset_name,
    }

    return input_arguments
