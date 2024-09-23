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

/// For all the events there is the time when it is scheduled to occur.
#[derive(PartialEq, Eq)]
enum Event {
    /// A new job arrives.
    JobStart(u64),
    /// The warm-up period expires.
    WarmupPeriodEnd(u64),
    /// The simulation ends.
    ExperimentEnd(u64),
    /// Print progress.
    Progress(u64, u16),
    /// A quantum iteration ends.
    QuantumIterationEnd(u64),
    /// Update classical tasks.
    UpdateClassicalTasks(u64),
}

impl Event {
    fn time(&self) -> u64 {
        match self {
            Self::JobStart(t)
            | Self::WarmupPeriodEnd(t)
            | Self::ExperimentEnd(t)
            | Self::Progress(t, _)
            | Self::QuantumIterationEnd(t)
            | Self::UpdateClassicalTasks(t) => *t,
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
    pub num_serverless_workers: usize,
    /// The number of quantum computers.
    pub num_quantum_computers: usize,
    /// The maximum queue length for classical tasks.
    pub max_classical_tasks: usize,
    /// The maximum queue length for quantum tasks.
    pub max_quantum_tasks: usize,
    /// The job type.
    pub job_type: String,
}

impl Config {
    pub fn header() -> String {
        "seed,duration,job_interarrival,warmup_period,worker_capacity,num_serverless_workers,num_quantum_computers,max_classical_tasks,max_quantum_tasks,job_type".to_string()
    }
    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{},{}",
            self.seed,
            self.duration,
            self.job_interarrival,
            self.warmup_period,
            self.worker_capacity,
            self.num_serverless_workers,
            self.num_quantum_computers,
            self.max_classical_tasks,
            self.max_quantum_tasks,
            self.job_type
        )
    }
}

pub struct Simulation {
    // internal data structures
    job_factory: crate::job::JobFactory,
    job_interarrival_rng: rand::rngs::StdRng,
    vqe_num_qubits_rng: rand::rngs::StdRng,
    active_jobs: std::collections::HashMap<u64, crate::job::Job>,
    active_classical_tasks: Vec<crate::task::Task>,
    pending_quantum_tasks: Vec<crate::task::Task>,
    active_quantum_tasks: Vec<crate::task::Task>,
    num_qubits: Vec<u16>,

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

        let mut tokens = config.job_type.split(';').clone().collect::<Vec<&str>>();
        anyhow::ensure!(!tokens.is_empty(), "invalid empty job type");
        anyhow::ensure!(tokens[0].to_ascii_lowercase() == "vqe", "invalid job type");
        anyhow::ensure!(
            tokens.len() > 1,
            "too few qubits specified for VQE job type"
        );
        tokens.remove(0);
        let num_qubits = tokens
            .iter()
            .filter_map(|x| x.parse::<u16>().ok())
            .collect::<Vec<u16>>();
        anyhow::ensure!(
            tokens.len() == num_qubits.len(),
            "cannot parse number of qubits in VQE job type"
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
            active_jobs: std::collections::HashMap::new(),
            active_classical_tasks: vec![],
            pending_quantum_tasks: vec![],
            active_quantum_tasks: vec![],
            num_qubits,
            config,
        })
    }

    /// Run a simulation.
    pub fn run(&mut self) -> crate::output::Output {
        // outputs
        let mut single = crate::output::OutputSingle::new();
        let mut series = crate::output::OutputSeries::new();

        // create the event queue and push initial events
        let mut events = std::collections::BinaryHeap::new();
        events.push(Event::JobStart(0));
        events.push(Event::WarmupPeriodEnd(to_nanoseconds(
            self.config.warmup_period,
        )));
        events.push(Event::ExperimentEnd(to_nanoseconds(self.config.duration)));
        for i in 1..100 {
            events.push(Event::Progress(
                to_nanoseconds(i as f64 * self.config.duration / 100.0),
                i,
            ));
        }

        // initialize simulated time and ID of the first job
        let mut now;

        // configure random variables for workload generation
        let job_interarrival_rv = rand_distr::Exp::new(1.0 / self.config.job_interarrival).unwrap();

        // metrics
        let mut num_job_accepted = 0;
        let mut num_job_dropped = 0;
        series.set_header("job_time", "num_qubits,priority");
        series.set_header("qc_iter_dur", "num_qubits,priority");
        series.set_header("classical_dur", "num_qubits,priority");

        // simulation loop
        let real_now = std::time::Instant::now();
        'main_loop: loop {
            if let Some(event) = events.pop() {
                now = event.time();
                match event {
                    Event::JobStart(time_arrival) => {
                        assert_eq!(time_arrival, now);

                        if self.active_classical_tasks.len() < self.config.max_classical_tasks
                            && self.pending_quantum_tasks.len() < self.config.max_quantum_tasks
                        {
                            // create a new job and draw randomly its lifetime
                            let num_qubits = self
                                .num_qubits
                                .choose(&mut self.vqe_num_qubits_rng)
                                .unwrap();
                            let job = self
                                .job_factory
                                .make(crate::job::JobType::Vqe(*num_qubits), now);
                            log::debug!("A {} {:?}", now, job);

                            // manage the job's initial task
                            if let Ok(mut job) = job {
                                if let Some(event) =
                                    self.manage_task(now, job.next_task(now).unwrap(), &mut single)
                                {
                                    events.push(event);
                                }

                                // add the job the map of active ones
                                self.active_jobs.insert(job.job_id, job);
                            } else {
                                log::warn!("error when creating a job with {} qubits", num_qubits);
                            }
                            num_job_accepted += 1;
                        } else {
                            num_job_dropped += 1;
                        }

                        // schedule a new job
                        events.push(Event::JobStart(
                            now + to_nanoseconds(
                                job_interarrival_rv.sample(&mut self.job_interarrival_rng),
                            ),
                        ));
                    }
                    Event::WarmupPeriodEnd(_) => {
                        log::debug!("W {}", now);
                        single.enable(now);
                        series.enable();
                    }
                    Event::ExperimentEnd(_) => {
                        log::debug!("E {}", now);
                        break 'main_loop;
                    }
                    Event::Progress(_, percentage) => {
                        assert!(
                            self.active_jobs.len()
                                == (self.active_classical_tasks.len()
                                    + self.active_quantum_tasks.len()
                                    + self.pending_quantum_tasks.len())
                        );
                        log::info!("completed {}% ({} active jobs, {} classical tasks, {}/{} quantum tasks", percentage, self.active_jobs.len(), self.active_classical_tasks.len(), self.active_quantum_tasks.len(), self.pending_quantum_tasks.len());
                    }
                    Event::QuantumIterationEnd(_) => {
                        self.log_internals("Q", now);

                        // find the completed task
                        assert!(!self.active_quantum_tasks.is_empty());
                        let pos = self
                            .active_quantum_tasks
                            .iter()
                            .position(|task| {
                                if let crate::task::TaskType::Quantum(residual) = task.task_type {
                                    residual == (now - task.last_update)
                                } else {
                                    false
                                }
                            })
                            .unwrap();
                        let completed_task = self.active_quantum_tasks.swap_remove(pos);
                        single.time_avg(
                            "active_quantum_tasks",
                            now,
                            self.active_quantum_tasks.len() as f64,
                        );
                        series.add(
                            "qc_iter_dur",
                            &self.active_jobs.get(&completed_task.job_id).unwrap().label,
                            to_seconds(now - completed_task.start_time),
                        );

                        let new_task_res = self.new_task_for_job(
                            now,
                            completed_task.job_id,
                            &mut series,
                            &mut single,
                        );
                        if new_task_res.0 {
                            let res = self.active_jobs.remove(&completed_task.job_id);
                            assert!(res.is_some());
                        }
                        if let Some(event) = new_task_res.1 {
                            events.push(event);
                        }

                        // if there is at least one pending quantum task put
                        // it into action
                        if let Some(mut new_task) = self.pending_quantum_tasks.pop() {
                            new_task.last_update = now;
                            if let crate::task::TaskType::Quantum(duration) = new_task.task_type {
                                events.push(Event::QuantumIterationEnd(now + duration));
                            }
                            self.active_quantum_tasks.push(new_task);
                            single.time_avg(
                                "pending_quantum_tasks",
                                now,
                                self.pending_quantum_tasks.len() as f64,
                            );
                            single.time_avg(
                                "active_quantum_tasks",
                                now,
                                self.active_quantum_tasks.len() as f64,
                            );
                        }
                    }
                    Event::UpdateClassicalTasks(_) => {
                        self.log_internals("C", now);

                        // count the active tasks since the last update
                        let num_tasks = self
                            .active_classical_tasks
                            .iter()
                            .map(|x| if x.last_update == now { 0 } else { 1 })
                            .sum::<u64>();
                        assert!(num_tasks <= self.active_classical_tasks.len() as u64);

                        // processing capacity during the last period, in ops/s
                        let capacity = if num_tasks == 0 {
                            None
                        } else {
                            Some(std::cmp::min(
                                self.config.worker_capacity,
                                self.config.num_serverless_workers as u64
                                    * self.config.worker_capacity
                                    / num_tasks,
                            ))
                        };

                        // update the residual of all the tasks
                        // and find which tasks are complete (if any)
                        let mut residuals = vec![];
                        let mut finished_tasks_start_times = vec![];
                        let mut finished_task_job_ids = std::collections::HashSet::new();
                        let capacity_ratio = capacity.map(|capacity| capacity as f64 / 1e9_f64);
                        for task in &mut self.active_classical_tasks {
                            let num_ops = if let Some(capacity_ratio) = capacity_ratio {
                                ((now - task.last_update) as f64 * capacity_ratio).ceil() as u64
                            } else {
                                0
                            };
                            task.last_update = now;
                            if let crate::task::TaskType::Classical(residual) = &mut task.task_type
                            {
                                assert!(*residual >= num_ops);
                                *residual -= num_ops;
                                if *residual == 0 {
                                    finished_tasks_start_times.push((task.job_id, task.start_time));
                                    finished_task_job_ids.insert(task.job_id);
                                } else {
                                    residuals.push(*residual);
                                }
                            }
                        }

                        if !residuals.is_empty() {
                            // find the smallest residual, if there tasks that
                            // are still active after this event is fully handled
                            residuals.sort_unstable();
                            let smallest_residual = residuals.first().unwrap();

                            // create an event that is handled when the task with
                            // the smallest residual finishes, unless there are new
                            // tasks arriving that will mess the schedule
                            events.push(Event::UpdateClassicalTasks(now + smallest_residual));
                        }

                        // add a performance sample for the task duration
                        for (job_id, start_time) in finished_tasks_start_times {
                            series.add(
                                "classical_dur",
                                &self.active_jobs.get(&job_id).unwrap().label,
                                to_seconds(now - start_time),
                            );
                        }

                        // remove the completed tasks from the active set
                        self.active_classical_tasks
                            .retain(|x| !finished_task_job_ids.contains(&x.job_id));
                        single.time_avg(
                            "active_classical_tasks",
                            now,
                            self.active_classical_tasks.len() as f64,
                        );

                        // for all jobs that are still active, schedule the
                        // next task, otherwise remove the job from the active set
                        for job_id in &finished_task_job_ids {
                            let new_task_res =
                                self.new_task_for_job(now, *job_id, &mut series, &mut single);
                            if new_task_res.0 {
                                let res = self.active_jobs.remove(job_id);
                                assert!(res.is_some());
                            }
                            if let Some(event) = new_task_res.1 {
                                events.push(event);
                            }
                        }
                    }
                }
            }
        }

        // save final metrics
        single.one_time("execution_time", real_now.elapsed().as_secs_f64());
        single.one_time("num_job_accepted", num_job_accepted as f64);
        single.one_time("num_job_dropped", num_job_dropped as f64);

        // return the simulation output
        crate::output::Output {
            single,
            series,
            config_csv: self.config.to_csv(),
        }
    }

    fn log_internals(&self, hdr: &str, now: u64) {
        log::debug!(
            "{} {} active jobs [{}] {:?}",
            hdr,
            now,
            self.active_jobs.len(),
            self.active_jobs
        );
        log::debug!(
            "{} {} classical tasks [{}] {:?}",
            hdr,
            now,
            self.active_classical_tasks.len(),
            self.active_classical_tasks
        );
        log::debug!(
            "{} {} pending quantum tasks [{}] {:?}",
            hdr,
            now,
            self.pending_quantum_tasks.len(),
            self.pending_quantum_tasks
        );
        log::debug!(
            "{} {} active quantum tasks [{}] {:?}",
            hdr,
            now,
            self.active_quantum_tasks.len(),
            self.active_quantum_tasks
        );
    }

    /// Return: boolean that is true if the job has to be removed, false otherwise;
    /// a new event to be scheduled.
    fn new_task_for_job(
        &mut self,
        now: u64,
        job_id: u64,
        series: &mut crate::output::OutputSeries,
        single: &mut crate::output::OutputSingle,
    ) -> (bool, Option<Event>) {
        let job = self.active_jobs.get_mut(&job_id).unwrap();
        if let Some(new_task) = job.next_task(now) {
            (false, self.manage_task(now, new_task, single))
        } else {
            series.add("job_time", &job.label, to_seconds(now - job.time_arrival));
            (true, None)
        }
    }

    fn manage_task(
        &mut self,
        now: u64,
        new_task: crate::task::Task,
        single: &mut crate::output::OutputSingle,
    ) -> Option<Event> {
        match &new_task.task_type {
            crate::task::TaskType::Classical(_residual) => {
                let event = Some(Event::UpdateClassicalTasks(now));
                self.active_classical_tasks.push(new_task);
                single.time_avg(
                    "active_classical_tasks",
                    now,
                    self.active_classical_tasks.len() as f64,
                );
                event
            }
            crate::task::TaskType::Quantum(duration) => {
                if self.active_quantum_tasks.len() < self.config.num_quantum_computers {
                    let event = Some(Event::QuantumIterationEnd(now + *duration));
                    self.active_quantum_tasks.push(new_task);
                    single.time_avg(
                        "active_quantum_tasks",
                        now,
                        self.active_quantum_tasks.len() as f64,
                    );
                    event
                } else {
                    self.pending_quantum_tasks.push(new_task);
                    single.time_avg(
                        "pending_quantum_tasks",
                        now,
                        self.pending_quantum_tasks.len() as f64,
                    );
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simulation_run() -> anyhow::Result<()> {
        Ok(())
    }
}
