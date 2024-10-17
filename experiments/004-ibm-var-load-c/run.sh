#!/bin/bash

num_serverless_workers_values="1 2 3 4 5 6"
job_interarrival_values="15 30 45 60 75 90"

bin=../../target/release/serverless_quantum_sim

ln -s ../../input . 2> /dev/null
mkdir data 2> /dev/null

for num_serverless_workers in $num_serverless_workers_values ; do
for job_interarrival in $job_interarrival_values ; do
  cmd="$bin \
    --duration $((24*3600)) \
    --warmup-period $((3600)) \
    --job-interarrival $job_interarrival \
    --num-serverless-workers $num_serverless_workers \
    --num-quantum-computers 100 \
    --max-classical-tasks 40 \
    --max-quantum-tasks 40 \
    --quantum-schedule-policy random \
    --job-type \"VQE;4;8;12;16;20;24\" \
    --priorities \"1\" \
    --concurrency 20 \
    --seed-init 0 \
    --seed-end 100 \
    --target-qc-dur-file ibm_job_estimate.csv \
    --append \
    "

    if [ "$DRY" != "" ] ; then
        echo $cmd
    else
        echo "job_interarrival $job_interarrival, num_quantum_computers $num_quantum_computers"
        eval $cmd
    fi

done
done
