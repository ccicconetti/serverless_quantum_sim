// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

use clap::Parser;
use std::io::Write;

#[derive(Debug, clap::Parser)]
#[command(long_about = None)]
struct Args {
    /// Duration of the simulation experiment, in s
    #[arg(long, default_value_t = 300_f64)]
    duration: f64,
    /// Duration of the warm-up period, in s
    #[arg(long, default_value_t = 30_f64)]
    warmup_period: f64,
    /// Average inter-arrival between consecutive jobs, in s
    #[arg(long, default_value_t = 60.0)]
    job_interarrival: f64,
    /// The capacity of each serverless worker, in operations/s
    #[arg(long, default_value_t = 1_000_000_000_u64)]
    worker_capacity: u64,
    /// The number of serverless workers
    #[arg(long, default_value_t = 4)]
    num_serverless_workers: usize,
    /// The number of quantum computers
    #[arg(long, default_value_t = 2)]
    num_quantum_computers: usize,
    /// The maximum queue length for classical tasks
    #[arg(long, default_value_t = 50)]
    max_classical_tasks: usize,
    /// The maximum queue length for quantum tasks
    #[arg(long, default_value_t = 50)]
    max_quantum_tasks: usize,
    /// The policy to schedule quantum tasks.
    #[arg(long, default_value_t = String::from("fifo"))]
    quantum_schedule_policy: String,
    /// The job type
    #[arg(long, default_value_t = String::from("VQE;4;6;8;10"))]
    job_type: String,
    /// The job priorities
    #[arg(long, default_value_t = String::from("1;2;4"))]
    priorities: String,
    /// Save iteration durations
    #[arg(long, default_value_t = false)]
    save_iteration_durations: bool,
    /// Print trace stats and quit
    #[arg(long, default_value_t = false)]
    trace_stats: bool,
    /// Initial seed to initialize the pseudo-random number generators
    #[arg(long, default_value_t = 0)]
    seed_init: u64,
    /// Final seed to initialize the pseudo-random number generators
    #[arg(long, default_value_t = 10)]
    seed_end: u64,
    /// Number of parallel workers
    #[arg(long, default_value_t = std::thread::available_parallelism().unwrap().get())]
    concurrency: usize,
    /// Name of the path where to save the metrics collected.
    #[arg(long, default_value_t = String::from("data/"))]
    output_path: String,
    /// Append to the output file.
    #[arg(long, default_value_t = false)]
    append: bool,
    /// Additional fields recorded in the CSV output file.
    #[arg(long, default_value_t = String::from(""))]
    additional_fields: String,
    /// Header of additional fields recorded in the CSV output file.
    #[arg(long, default_value_t = String::from(""))]
    additional_header: String,
}

fn open_output_file(
    path: &str,
    filename: &str,
    append: bool,
    header: &str,
) -> anyhow::Result<std::fs::File> {
    let output_single_filename = format!("{}{}", path, filename);
    let add_header = !append
        || match std::fs::metadata(&output_single_filename) {
            Ok(metadata) => metadata.len() == 0,
            Err(_) => true,
        };
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .append(append)
        .create(true)
        .truncate(!append)
        .open(output_single_filename)?;
    if add_header {
        writeln!(&mut f, "{}", header)?;
    }
    Ok(f)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    anyhow::ensure!(
        args.additional_fields.matches(',').count() == args.additional_header.matches(',').count(),
        "--additional_fields and --additional_header have a different number of commas"
    );

    // create the configurations of all the experiments
    let configurations = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    for seed in args.seed_init..args.seed_end {
        configurations
            .lock()
            .unwrap()
            .push(serverless_quantum_sim::simulation::Config {
                seed,
                duration: args.duration,
                job_interarrival: args.job_interarrival,
                warmup_period: args.warmup_period,
                worker_capacity: args.worker_capacity,
                num_serverless_workers: args.num_serverless_workers,
                num_quantum_computers: args.num_quantum_computers,
                max_classical_tasks: args.max_classical_tasks,
                max_quantum_tasks: args.max_quantum_tasks,
                quantum_schedule_policy: args.quantum_schedule_policy.clone(),
                job_type: args.job_type.clone(),
                priorities: args.priorities.clone(),
                save_iteration_durations: args.save_iteration_durations,
            });
    }

    if configurations.lock().unwrap().is_empty() {
        return Ok(());
    }

    if args.trace_stats {
        let trace_stats = serverless_quantum_sim::job::JobFactory::new(0)
            .unwrap()
            .trace_stats();
        let mut alt: std::collections::BTreeMap<u16, std::collections::HashMap<String, f64>> =
            std::collections::BTreeMap::new();
        for (elem, records) in trace_stats {
            println!("{}", elem);
            for record in records {
                println!(
                    "num_qubits {:>3} -> {} / {} / {}",
                    record.0, record.1, record.2, record.3
                );
                alt.entry(record.0)
                    .or_default()
                    .insert(elem.clone(), record.2);
            }
        }
        println!("average job times");
        for (num_qubits, values) in alt {
            let avg_job_time = values["pre"]
                + values["num_iterations"] * (values["iter"] + values["dur_qc"])
                + values["post"];
            println!("num_qubits {:>3} -> {}", num_qubits, avg_job_time);
        }
        return Ok(());
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    for i in 0..std::cmp::min(args.concurrency, (args.seed_end - args.seed_init) as usize) {
        let tx = tx.clone();
        let configurations = configurations.clone();
        tokio::spawn(async move {
            log::info!("spawned worker #{}", i);
            loop {
                let config;
                {
                    if let Some(val) = configurations.lock().unwrap().pop() {
                        config = Some(val);
                    } else {
                        break;
                    }
                }
                match serverless_quantum_sim::simulation::Simulation::new(config.unwrap()) {
                    Ok(mut sim) => tx.send(sim.run()).unwrap(),
                    Err(err) => log::error!("error when running simulation: {}", err),
                };
            }
            log::info!("terminated worker #{}", i);
        });
    }
    let _ = || tx;

    // wait until all the simulations have been done
    let mut outputs = vec![];
    while let Some(output) = rx.recv().await {
        outputs.push(output);
    }

    // save output to files
    assert!(!outputs.is_empty());
    let mut single_file = open_output_file(
        &args.output_path,
        "single.csv",
        args.append,
        format!(
            "{}{},{}",
            args.additional_header,
            serverless_quantum_sim::simulation::Config::header(),
            outputs.first().unwrap().single.header()
        )
        .as_str(),
    )?;

    for output in outputs {
        writeln!(
            &mut single_file,
            "{}{},{}",
            args.additional_fields,
            output.config_csv,
            output.single.to_csv()
        )?;

        for (name, elem) in &output.series.series {
            let mut series_file = open_output_file(
                &args.output_path,
                format!("{}.csv", name).as_str(),
                args.append,
                format!(
                    "{}{},{},value",
                    args.additional_header,
                    serverless_quantum_sim::simulation::Config::header(),
                    elem.header
                )
                .as_str(),
            )?;
            for (label, values) in &elem.values {
                for value in values {
                    writeln!(
                        &mut series_file,
                        "{}{},{},{}",
                        args.additional_fields, output.config_csv, label, value
                    )?;
                }
            }
        }
    }

    Ok(())
}
