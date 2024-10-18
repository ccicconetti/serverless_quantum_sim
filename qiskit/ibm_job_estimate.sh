#!/bin/bash

backends="ibm_torino ibm_brussels ibm_nazca"
n_qubits="4 6 8 10 14 18 22 26 30 40 50 60 70 80 90"

for b in $backends ; do
for n in $n_qubits ; do
    BACKEND=$b NUM_QUBITS=$n python ibm_job_estimate.py
done
done
