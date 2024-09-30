#!/bin/bash

num_serverless_workers_values="2 4 6 8 10"
job_interarrival_values="5 10 15 20 25"

bin=../../target/release/serverless_quantum_sim

ln -s ../../input . 2> /dev/null
mkdir data 2> /dev/null

for num_serverless_workers in $num_serverless_workers_values ; do
for job_interarrival in $job_interarrival_values ; do
  cmd="$bin \
    --duration 86400 \
    --warmup-period 3600 \
    --job-interarrival $job_interarrival \
    --num-serverless-workers $num_serverless_workers \
    --num-quantum-computers 2 \
    --max-classical-tasks 20 \
    --max-quantum-tasks 20 \
    --quantum-schedule-policy random \
    --job-type \"VQE;4;6;8;10\" \
    --priorities \"1\" \
    --concurrency 20 \
    --seed-init 0 \
    --seed-end 20 \
    --append
    "

    if [ "$DRY" != "" ] ; then
        echo $cmd
    else
        echo "job_interarrival $job_interarrival, num_serverless_workers $num_serverless_workers"
        eval $cmd
    fi

done
done