# Modified version of the example code provided by Qiskit serverless for VQE.
# The original code is available on GitHub:
# https://github.com/Qiskit/qiskit-serverless

import numpy as np
import os
import time
from qiskit_ibm_runtime import SamplerV2 as Sampler, QiskitRuntimeService
from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
from qiskit.circuit.library import EfficientSU2

from ibm_credentials import get_ibm_credentials
from prepare_input import prepare_input, get_datasets, dataset_name

NUM_QUBITS = int(os.environ.get("NUM_QUBITS", "4"))
OUTPUT = os.environ.get("OUTPUT", "ibm_job_estimate.csv")

# Find a dataset with the right number of qubits
datasets = get_datasets(min_qubits=NUM_QUBITS, max_qubits=NUM_QUBITS)
assert datasets != []
arguments = prepare_input(dataset_name(datasets[0]), maxiter=1, ibm_credentials=None)

# Check IBM credentials
ibm_credentials = get_ibm_credentials()

service = QiskitRuntimeService(
    channel=ibm_credentials["CHANNEL"],
    instance=ibm_credentials["INSTANCE"],
    token=ibm_credentials["TOKEN"],
)

ansatz = EfficientSU2(NUM_QUBITS)
backend = service.least_busy(operational=True, simulator=False)
backend_name = backend.name
pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
ansatz_isa = pm.run(ansatz)
operator_isa = arguments["operator"].apply_layout(ansatz_isa.layout)

initial_parameters = 2 * np.pi * np.random.rand(ansatz.num_parameters)
qc = ansatz.assign_parameters(initial_parameters)
qc.measure_all()
qc_isa = pm.run(qc)

sampler = Sampler(backend=backend)
job = sampler.run([qc_isa])

usage_estimation = job.usage_estimation["quantum_seconds"]

with open(OUTPUT, "a") as outfile:
    outfile.write(
        "{},{},{},{}\n".format(time.time(), NUM_QUBITS, backend_name, usage_estimation)
    )

job.cancel()
