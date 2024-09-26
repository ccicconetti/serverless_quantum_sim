#!/bin/bash

for (( n = 4 ; n <= 10 ; n += 2 )) ; do
    NUM_QUBITS=$n python ibm_job_estimate.py
done