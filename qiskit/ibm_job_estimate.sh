#!/bin/bash

for (( n = 4 ; n <= 90 ; n += 2 )) ; do
    NUM_QUBITS=$n python ibm_job_estimate.py
done
