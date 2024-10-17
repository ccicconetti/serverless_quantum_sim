#!/bin/bash

quantum_schedule_policies="fifo lifo random weighted"

bin=../../target/release/serverless_quantum_sim

ln -s ../../input . 2> /dev/null
mkdir data 2> /dev/null

for quantum_schedule_policy in $quantum_schedule_policies ; do
  cmd="$bin \
    --duration $((86400*7)) \
    --warmup-period $((3600*12)) \
    --job-interarrival 600 \
    --num-serverless-workers 1 \
    --num-quantum-computers 4 \
    --max-classical-tasks 40 \
    --max-quantum-tasks 40 \
    --quantum-schedule-policy $quantum_schedule_policy \
    --job-type \"VQE;4;6;8;10\" \
    --priorities \"1;2;4\" \
    --concurrency 20 \
    --seed-init 0 \
    --seed-end 100 \
    --target-qc-dur-file ibm_job_estimate.csv \
    --append
    "

    if [ "$DRY" != "" ] ; then
        echo $cmd
    else
        echo $quantum_schedule_policy
        eval $cmd
    fi

done
