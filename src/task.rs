// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

#[derive(Debug)]
pub struct Task {
    /// Job identifier.
    job_id: u64,
    /// Start time, in ns.
    start_time: u64,
}
