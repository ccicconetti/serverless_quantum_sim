// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

use rand::{distributions::Distribution, SeedableRng};

const MILLI_SECOND: u64 = 1000000000;
const SECOND: u64 = MILLI_SECOND * 1000;

#[derive(Debug)]
pub enum JobType {
    /// Variational Quantum Eigensolver with variable number of qubits.
    Vqe(usize),
}

#[derive(Debug)]
pub struct Job {
    /// Job type.
    job_type: JobType,
    /// Numeric application identifier.
    job_id: u64,
    /// Number of operations for the preparation phase.
    num_operations_pre: u64,
    /// Number of operations for each iteration.
    num_operations_iter: u64,
    /// Number of operations for the post-processing phase.
    num_operations_post: u64,
    /// Time to execute a single iteration on a QC, in ns.
    dur_qc_iteration: u64,
    /// Number of iterations.
    num_iterations: u64,
    /// Arrival time, in ns.
    time_arrival: u64,
}

pub struct JobFactory {
    /// Random distribution to determine the number of iterations
    rv_num_iterations: rand::distributions::Uniform<f64>,
    /// Random distribution to determine the number of classical PRE operations
    rv_num_operations_pre: rand::distributions::Uniform<f64>,
    /// Random distribution to determine the number of classical operations
    /// at each iteration
    rv_num_operations_iter: rand::distributions::Uniform<f64>,
    /// Random distribution to determine the number of classical POST operations
    rv_num_operations_post: rand::distributions::Uniform<f64>,
    /// Random distribution to determine the time to execute a QC iteration, in ns
    rv_dur_qc_iteration: rand::distributions::Uniform<f64>,
    /// RNG
    rng: rand::rngs::StdRng,
    /// Next job ID.
    next_job_id: u64,
}

impl JobFactory {
    /// Create a factory of jobs.
    /// Parameters:
    /// - `seed`: pseudo-random number generator seed
    pub fn new(seed: u64) -> anyhow::Result<Self> {
        let rv_num_iterations = rand::distributions::uniform::Uniform::new(0_f64, 1_f64);
        let rv_num_operations_pre = rand::distributions::uniform::Uniform::new(0_f64, 1_f64);
        let rv_num_operations_iter = rand::distributions::uniform::Uniform::new(0_f64, 1_f64);
        let rv_num_operations_post = rand::distributions::uniform::Uniform::new(0_f64, 1_f64);
        let rv_dur_qc_iteration = rand::distributions::uniform::Uniform::new(0_f64, 1_f64);
        let mut seed_cnt = 0_u64;
        let mut next_seed = || {
            seed_cnt += 1;
            seed + 1000000 * seed_cnt
        };

        Ok(Self {
            rv_num_iterations,
            rv_num_operations_pre,
            rv_num_operations_iter,
            rv_num_operations_post,
            rv_dur_qc_iteration,
            rng: rand::rngs::StdRng::seed_from_u64(next_seed()),
            next_job_id: 0,
        })
    }

    /// Create a new random job.
    /// Parameters:
    /// - `job_type`: the job type
    /// - `time_arrival`: the time of arrival of this job, in ns
    pub fn make(&mut self, job_type: JobType, time_arrival: u64) -> Job {
        let id = self.next_job_id;
        self.next_job_id += 1;

        let to_u64 = |x: f64, a: u64, b: u64| a + ((b - a) as f64 * x) as u64;

        match job_type {
            JobType::Vqe(num_qubits) => {
                let num_operations_pre =
                    to_u64(self.rv_num_operations_pre.sample(&mut self.rng), 100, 1000);
                let num_operations_iter =
                    to_u64(self.rv_num_operations_iter.sample(&mut self.rng), 100, 1000);
                let num_operations_post =
                    to_u64(self.rv_num_operations_post.sample(&mut self.rng), 100, 1000);
                let dur_qc_iteration = to_u64(
                    self.rv_dur_qc_iteration.sample(&mut self.rng),
                    MILLI_SECOND,
                    10 * MILLI_SECOND,
                );
                let num_iterations = to_u64(self.rv_num_iterations.sample(&mut self.rng), 10, 100);

                Job {
                    job_type,
                    job_id: id,
                    num_operations_pre,
                    num_operations_iter,
                    num_operations_post,
                    dur_qc_iteration,
                    num_iterations,
                    time_arrival,
                }
            }
        }
    }
}
