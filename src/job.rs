// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

use average::{concatenate, Estimate, Max, Mean, Min};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::io::BufRead;

const MILLISECOND: u64 = 1_000_000;
const SECOND: u64 = 1_000 * MILLISECOND;

#[derive(Debug)]
pub enum JobType {
    /// Variational Quantum Eigensolver with variable number of qubits
    /// read from traces.
    Vqe(u16),
}

#[derive(Debug)]
pub enum JobStatus {
    Preparation,
    ClassicalIteration(u64),
    QuantumIteration(u64),
    Postprocessing,
    Completed,
}

#[derive(Debug)]
pub struct Job {
    #[allow(dead_code)]
    /// Job type.
    job_type: JobType,
    /// Job status.
    job_status: JobStatus,
    /// Numeric application identifier.
    pub job_id: u64,
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
    pub time_arrival: u64,
    /// Number of qubits.
    pub num_qubits: u16,
    /// Priority (a higher value means a higher priority).
    pub priority: u16,
    /// Label.
    pub label: String,
}

impl Job {
    pub fn next_task(&mut self, cur_time: u64) -> Option<crate::task::Task> {
        let task_type = match &self.job_status {
            JobStatus::Preparation => {
                self.job_status = JobStatus::ClassicalIteration(1);
                crate::task::TaskType::Classical(self.num_operations_pre)
            }
            JobStatus::ClassicalIteration(num_iteration) => {
                self.job_status = JobStatus::QuantumIteration(*num_iteration);
                crate::task::TaskType::Classical(self.num_operations_iter)
            }
            JobStatus::QuantumIteration(num_iteration) => {
                if *num_iteration == self.num_iterations {
                    self.job_status = JobStatus::Postprocessing;
                } else {
                    self.job_status = JobStatus::ClassicalIteration(*num_iteration + 1);
                }
                crate::task::TaskType::Quantum(self.dur_qc_iteration)
            }
            JobStatus::Postprocessing => {
                self.job_status = JobStatus::Completed;
                crate::task::TaskType::Classical(self.num_operations_post)
            }
            JobStatus::Completed => {
                return None;
            }
        };
        Some(crate::task::Task {
            job_id: self.job_id,
            task_type,
            start_time: cur_time,
            last_update: cur_time,
        })
    }
}

pub struct JobFactory {
    /// RNG
    rng: rand::rngs::StdRng,
    /// Next job ID.
    next_job_id: u64,
    /// Possibile number of operations for the preparation phase.
    pre_values: std::collections::HashMap<u16, Vec<u64>>,
    /// Possibile number of operations for the iteration phase.
    iter_values: std::collections::HashMap<u16, Vec<u64>>,
    /// Possibile number of operations for the post-processing phase.
    post_values: std::collections::HashMap<u16, Vec<u64>>,
    /// Possibile durations of the QC iterations, in ns.
    dur_qc_values: std::collections::HashMap<u16, Vec<u64>>,
    /// Possibile number of iteration values.
    num_iterations_values: std::collections::HashMap<u16, Vec<u64>>,
}

concatenate!(Estimator, [Min, min], [Max, max], [Mean, mean]);

impl JobFactory {
    /// Create a factory of jobs.
    /// Parameters:
    /// - `seed`: pseudo-random number generator seed
    /// - `target_dur_qc_avg`: target durations, in s, of the quantum iterations
    ///   can be empty for some or all values, in which case there is no
    ///   adjustment of the values read from the trace file
    pub fn new(
        seed: u64,
        target_dur_qc_avg: &std::collections::BTreeMap<u16, f64>,
    ) -> anyhow::Result<Self> {
        let input_files = std::collections::HashMap::from([
            ("pre", "input/pre.csv"),
            ("iter", "input/cost_time.csv"),
            ("post", "input/post.csv"),
            ("dur_qc_values", "input/exec_time.csv"),
            ("num_iterations", "input/num_iterations.csv"),
        ]);

        // Check that all the required input files exist.
        let mut non_existing_files = vec![];
        for input_filename in input_files.values() {
            if !std::path::Path::new(input_filename).exists() {
                non_existing_files.push(input_filename.to_string());
            }
        }
        anyhow::ensure!(
            non_existing_files.is_empty(),
            format!("missing input files: {}", non_existing_files.join(","))
        );

        let pre_values = Self::read_from_file(input_files["pre"], SECOND as f64)?;
        let iter_values = Self::read_from_file(input_files["iter"], SECOND as f64)?;
        let post_values = Self::read_from_file(input_files["post"], SECOND as f64)?;
        let mut dur_qc_values = Self::read_from_file(input_files["dur_qc_values"], SECOND as f64)?;
        let num_iterations_values = Self::read_from_file(input_files["num_iterations"], 1_f64)?;

        let dur_qc_stats = Self::single_trace_stats(1.0 / SECOND as f64, &dur_qc_values);
        for (num_qubits, values) in &mut dur_qc_values {
            let average = dur_qc_stats.iter().find(|x| x.0 == *num_qubits).unwrap().2;
            if let Some(target_average) = target_dur_qc_avg.get(num_qubits) {
                for value in values {
                    *value = (*value as f64 * (*target_average / average)).round() as u64;
                }
            }
        }

        Ok(Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            next_job_id: 0,
            pre_values,
            iter_values,
            post_values,
            dur_qc_values,
            num_iterations_values,
        })
    }

    fn single_trace_stats(
        multiplier: f64,
        data: &std::collections::HashMap<u16, Vec<u64>>,
    ) -> Vec<(u16, f64, f64, f64)> {
        let mut estimators: std::collections::BTreeMap<u16, Estimator> =
            std::collections::BTreeMap::new();
        for (k, values) in data {
            let estimator = estimators.entry(*k).or_default();
            for value in values {
                estimator.add(*value as f64 * multiplier);
            }
        }
        let mut ret = vec![];
        for (k, estimator) in estimators {
            ret.push((k, estimator.min(), estimator.mean(), estimator.max()))
        }
        ret
    }

    pub fn trace_stats(&self) -> std::collections::HashMap<String, Vec<(u16, f64, f64, f64)>> {
        let mut ret = std::collections::HashMap::new();
        ret.insert(
            "pre".to_string(),
            JobFactory::single_trace_stats(1.0 / SECOND as f64, &self.pre_values),
        );
        ret.insert(
            "iter".to_string(),
            JobFactory::single_trace_stats(1.0 / SECOND as f64, &self.iter_values),
        );
        ret.insert(
            "post".to_string(),
            JobFactory::single_trace_stats(1.0 / SECOND as f64, &self.post_values),
        );
        ret.insert(
            "dur_qc".to_string(),
            JobFactory::single_trace_stats(1.0 / SECOND as f64, &self.dur_qc_values),
        );
        ret.insert(
            "num_iterations".to_string(),
            JobFactory::single_trace_stats(1.0 as f64, &self.num_iterations_values),
        );
        ret
    }

    fn read_from_file(
        filename: &str,
        multiplier: f64,
    ) -> anyhow::Result<std::collections::HashMap<u16, Vec<u64>>> {
        let mut res = std::collections::HashMap::new();

        let file = std::fs::File::open(filename)?;
        let reader = std::io::BufReader::new(file);

        for line in reader.lines().map_while(Result::ok) {
            let tokens = line.split(',').collect::<Vec<&str>>();
            anyhow::ensure!(
                tokens.len() == 2,
                "invalid line from file '{}': {}",
                filename,
                line
            );
            if let (Ok(num_qubits), Ok(value)) =
                (tokens[0].parse::<u16>(), tokens[1].parse::<f64>())
            {
                res.entry(num_qubits)
                    .or_insert(vec![])
                    .push((value * multiplier).round() as u64);
            }
        }
        Ok(res)
    }

    /// Create a new random job.
    /// Parameters:
    /// - `job_type`: the job type
    /// - `priority`: the job priority
    /// - `time_arrival`: the time of arrival of this job, in ns
    pub fn make(
        &mut self,
        job_type: JobType,
        priority: u16,
        time_arrival: u64,
    ) -> anyhow::Result<Job> {
        let id = self.next_job_id;
        self.next_job_id += 1;

        match job_type {
            JobType::Vqe(num_qubits) => {
                let num_operations_pre = if let Some(values) = self.pre_values.get(&num_qubits) {
                    values.choose(&mut self.rng).unwrap()
                } else {
                    anyhow::bail!(
                        "number of qubits not found in preparation phase trace: {}",
                        num_qubits
                    )
                };
                let num_operations_iter = if let Some(values) = self.iter_values.get(&num_qubits) {
                    values.choose(&mut self.rng).unwrap()
                } else {
                    anyhow::bail!(
                        "number of qubits not found in classical iteration trace: {}",
                        num_qubits
                    )
                };
                let num_operations_post = if let Some(values) = self.post_values.get(&num_qubits) {
                    values.choose(&mut self.rng).unwrap()
                } else {
                    anyhow::bail!(
                        "number of qubits not found in post-processing phase trace: {}",
                        num_qubits
                    )
                };
                let dur_qc_iteration = if let Some(values) = self.dur_qc_values.get(&num_qubits) {
                    values.choose(&mut self.rng).unwrap()
                } else {
                    anyhow::bail!(
                        "number of qubits not found in QC execution trace: {}",
                        num_qubits
                    )
                };
                let num_iterations =
                    if let Some(values) = self.num_iterations_values.get(&num_qubits) {
                        values.choose(&mut self.rng).unwrap()
                    } else {
                        anyhow::bail!(
                            "number of qubits not found in number of iterations trace: {}",
                            num_qubits
                        )
                    };

                Ok(Job {
                    job_type,
                    job_status: JobStatus::Preparation,
                    job_id: id,
                    num_operations_pre: *num_operations_pre,
                    num_operations_iter: *num_operations_iter,
                    num_operations_post: *num_operations_post,
                    dur_qc_iteration: *dur_qc_iteration,
                    num_iterations: *num_iterations,
                    time_arrival,
                    num_qubits,
                    priority,
                    label: format!("{},{}", num_qubits, priority),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_factory() -> anyhow::Result<()> {
        let mut jf = JobFactory::new(42, &std::collections::BTreeMap::new()).unwrap();
        let num_qubits_choices = vec![4, 6, 8, 10];
        let mut id = 0;
        for i in 0..10 {
            for num_qubits in &num_qubits_choices {
                let job = jf
                    .make(JobType::Vqe(*num_qubits), i, i as u64 * 1000)
                    .unwrap();
                println!("{:?}", job);
                assert!(job.num_operations_pre > 0);
                assert!(job.num_operations_iter > 0);
                assert!(job.num_operations_post > 0);
                assert!(job.dur_qc_iteration > 0);
                assert!(job.num_iterations > 0);
                assert!(job.num_iterations < 1000000);
                assert_eq!(i, job.priority);
                assert_eq!(i as u64 * 1000, job.time_arrival);
                assert_eq!(id, job.job_id);
                id += 1;
            }
        }

        assert!(jf.make(JobType::Vqe(999), 0, 0).is_err());

        Ok(())
    }
}
