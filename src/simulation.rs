// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_distr::Distribution;

static GIGA: u64 = 1000000000;

fn to_seconds(ns: u64) -> f64 {
    ns as f64 / GIGA as f64
}

fn to_nanoseconds(s: f64) -> u64 {
    (s * GIGA as f64).round() as u64
}

pub struct OutputSingle {
    /// Real execution time, in s.
    execution_time: f64,
}

impl OutputSingle {
    pub fn header() -> String {
        "execution_time".to_string()
    }
    pub fn to_csv(&self) -> String {
        format!("{}", self.execution_time)
    }
}

pub struct OutputSeries {
    pub series: std::collections::HashMap<String, Vec<f64>>,
}

pub struct Output {
    pub single: OutputSingle,
    pub series: OutputSeries,
    pub config_csv: String,
}

#[derive(PartialEq, Eq)]
enum Event {
    /// A new job arrives.
    /// 0: Event time.
    JobStart(u64),
    /// The warm-up period expires.
    /// 0: Event time.
    WarmupPeriodEnd(u64),
    /// The simulation ends.
    /// 0: Event time.
    ExperimentEnd(u64),
}

impl Event {
    fn time(&self) -> u64 {
        match self {
            Self::JobStart(t) | Self::WarmupPeriodEnd(t) | Self::ExperimentEnd(t) => *t,
        }
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.time().partial_cmp(&self.time())
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Debug)]
pub struct Config {
    /// The seed to initialize pseudo-random number generators.
    pub seed: u64,
    /// The duration of the simulation, in s.
    pub duration: f64,
    /// The average interval between two jobs, in s.
    pub job_interarrival: f64,
    /// The warm-up period, in s.
    pub warmup_period: f64,
    /// The capacity of each serverless worker, in operations/s.
    pub worker_capacity: u64,
    /// The number of serverless workers.
    pub num_serverless_workers: u64,
    /// The number of quantum computers.
    pub num_quantum_computers: u64,
}

impl Config {
    pub fn header() -> String {
        "seed,duration,job_interarrival,warmup_period,worker_capacity,num_serverless_workers,num_quantum_computers".to_string()
    }
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{}",
            self.seed,
            self.duration,
            self.job_interarrival,
            self.warmup_period,
            self.worker_capacity,
            self.num_serverless_workers,
            self.num_quantum_computers
        )
    }
}

pub struct Simulation {
    // internal data structures
    job_factory: crate::job::JobFactory,
    job_interarrival_rng: rand::rngs::StdRng,
    vqe_num_qubits_rng: rand::rngs::StdRng,
    active_classical: Vec<crate::task::Task>,
    active_quantum: Vec<crate::task::Task>,

    // configuration
    config: Config,
}

impl Simulation {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        anyhow::ensure!(config.duration > 0.0, "vanishing duration");
        anyhow::ensure!(
            config.job_interarrival > 0.0,
            "vanishing avg job interarrival time"
        );

        let mut seed_cnt = 0_u64;
        let mut next_seed = || {
            seed_cnt += 1;
            config.seed + 1000000 * seed_cnt
        };

        Ok(Self {
            job_factory: crate::job::JobFactory::new(config.seed)?,
            job_interarrival_rng: rand::rngs::StdRng::seed_from_u64(next_seed()),
            vqe_num_qubits_rng: rand::rngs::StdRng::seed_from_u64(next_seed()),
            active_classical: vec![],
            active_quantum: vec![],
            config,
        })
    }

    /// Run a simulation.
    pub fn run(&mut self) -> Output {
        // create the event queue and push initial events
        let mut events = std::collections::BinaryHeap::new();
        events.push(Event::JobStart(0));
        events.push(Event::WarmupPeriodEnd(to_nanoseconds(
            self.config.warmup_period,
        )));
        events.push(Event::ExperimentEnd(to_nanoseconds(self.config.duration)));

        // initialize simulated time and ID of the first job
        let mut now = 0;

        // configure random variables for workload generation
        let job_interarrival_rv = rand_distr::Exp::new(1.0 / self.config.job_interarrival).unwrap();
        let vqe_num_qubits_choices: Vec<usize> = vec![4, 6, 8, 10];

        // simulation loop
        let real_now = std::time::Instant::now();
        let mut warmup = true;
        'main_loop: loop {
            if let Some(event) = events.pop() {
                now = event.time();
                match event {
                    Event::JobStart(time_arrival) => {
                        assert_eq!(time_arrival, now);
                        // create a new job and draw randomly its lifetime
                        let job = self.job_factory.make(
                            crate::job::JobType::Vqe(
                                *vqe_num_qubits_choices
                                    .choose(&mut self.vqe_num_qubits_rng)
                                    .unwrap(),
                            ),
                            now,
                        );
                        log::debug!("A {} {:?}", now, job);

                        // schedule a new job
                        events.push(Event::JobStart(
                            now + to_nanoseconds(
                                job_interarrival_rv.sample(&mut self.job_interarrival_rng),
                            ),
                        ));
                    }
                    Event::WarmupPeriodEnd(_) => {
                        warmup = false;
                    }
                    Event::ExperimentEnd(_) => {
                        log::debug!("E {}", now);
                        break 'main_loop;
                    }
                }
            }
        }
        let execution_time = real_now.elapsed().as_secs_f64();

        // return the simulation output
        let series = std::collections::HashMap::new();
        Output {
            single: OutputSingle { execution_time },
            series: OutputSeries { series },
            config_csv: self.config.to_csv(),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_simulation_run() -> anyhow::Result<()> {
        Ok(())
    }
}
