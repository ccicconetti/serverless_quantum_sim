// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

#[derive(Debug, Clone, Copy)]
pub enum TaskType {
    /// Classical task, with specified residual number of operations.
    Classical(u64),
    /// Quantum task, with specified residual time of execution, in ns.
    Quantum(u64),
}

#[derive(Debug)]
pub struct Task {
    /// Job identifier.
    pub job_id: u64,
    /// Task type.
    pub task_type: TaskType,
    /// Start time, in ns.
    pub start_time: u64,
    /// Last update time, in ns.
    pub last_update: u64,
}
