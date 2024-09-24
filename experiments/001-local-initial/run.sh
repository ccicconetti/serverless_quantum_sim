#!/bin/bash

quantum_schedule_policies="fifo lifo random weighted"
quantum_schedule_policies="lifo"

bin=../../target/release/serverless_quantum_sim

ln -s ../../input . 2> /dev/null
mkdir data 2> /dev/null

for quantum_schedule_policy in $quantum_schedule_policies ; do
  cmd="$bin \
    --duration 3600 \
    --warmup-period 360 \
    --job-interarrival 15 \
    --num-serverless-workers 10 \
    --num-quantum-computers 2 \
    --max-classical-tasks 20 \
    --max-quantum-tasks 20 \
    --quantum-schedule-policy $quantum_schedule_policy \
    --job-type \"VQE;4;6;8;10\" \
    --priorities \"1;2;4\" \
    --concurrency 20 \
    --seed-init 0 \
    --seed-end 10 \
    --append
    "

    if [ "$DRY" != "" ] ; then
        echo $cmd
    else
        echo $quantum_schedule_policy
        eval $cmd
    fi

done