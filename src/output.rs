// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-License-Identifier: MIT

struct TimeAvg {
    last_update: u64,
    last_value: f64,
    sum_values: f64,
    sum_time: f64,
}

impl TimeAvg {
    pub fn new(last_update: u64) -> Self {
        Self {
            last_update,
            last_value: 0.0,
            sum_values: 0.0,
            sum_time: 0.0,
        }
    }
    pub fn avg(&self) -> f64 {
        self.sum_values / self.sum_time
    }
}

pub struct OutputSingle {
    enabled: bool,
    warmup: u64,
    one_time: std::collections::BTreeMap<String, f64>,
    time_avg: std::collections::BTreeMap<String, TimeAvg>,
}

impl OutputSingle {
    pub fn new() -> Self {
        Self {
            enabled: false,
            warmup: 0,
            one_time: std::collections::BTreeMap::new(),
            time_avg: std::collections::BTreeMap::new(),
        }
    }

    pub fn one_time(&mut self, name: &str, value: f64) {
        if self.enabled {
            self.one_time.insert(name.to_string(), value);
        }
    }

    pub fn time_avg(&mut self, name: &str, now: u64, value: f64) {
        let entry = self
            .time_avg
            .entry(name.to_string())
            .or_insert_with(|| TimeAvg::new(self.warmup));
        if self.enabled {
            let delta = (now - entry.last_update) as f64;
            entry.sum_values += delta * entry.last_value;
            entry.sum_time += delta;
            entry.last_update = now;
        }
        entry.last_value = value;
    }

    pub fn header(&self) -> String {
        format!(
            "{},{}",
            self.one_time
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(","),
            self.time_avg
                .keys()
                .cloned()
                .collect::<Vec<String>>()
                .join(",")
        )
    }
    pub fn to_csv(&self) -> String {
        format!(
            "{},{}",
            self.one_time
                .values()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(","),
            self.time_avg
                .values()
                .map(|x| x.avg().to_string())
                .collect::<Vec<String>>()
                .join(",")
        )
    }

    pub fn enable(&mut self, now: u64) {
        self.enabled = true;
        self.warmup = now;
        for elem in &mut self.time_avg.values_mut() {
            elem.last_update = now;
        }
    }

    pub fn finish(&mut self, now: u64) {
        for entry in &mut self.time_avg.values_mut() {
            let delta = (now - entry.last_update) as f64;
            entry.sum_values += delta * entry.last_value;
            entry.sum_time += delta;
            entry.last_update = now;
        }
    }
}

impl Default for OutputSingle {
    fn default() -> Self {
        Self::new()
    }
}

pub struct OutputSeriesSingle {
    pub header: String,
    pub values: std::collections::HashMap<String, Vec<f64>>,
}

impl Default for OutputSeriesSingle {
    fn default() -> Self {
        Self {
            header: "label".to_string(),
            values: std::collections::HashMap::new(),
        }
    }
}

/// Series of values.
/// The values are not recorded until `enabled()` is called.
/// Each series is associated with a name (with optional header) and a label.
pub struct OutputSeries {
    enabled: bool,
    pub series: std::collections::HashMap<String, OutputSeriesSingle>,
}

impl Default for OutputSeries {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputSeries {
    pub fn new() -> Self {
        Self {
            enabled: false,
            series: std::collections::HashMap::new(),
        }
    }

    /// Add a new value to a series metric.
    /// Parameters:
    /// - `name`: the metric name.
    /// - `label`: a label associated with the value.
    /// - `value`: the value added, if collection is enabled.
    pub fn add(&mut self, name: &str, label: &str, value: f64) {
        if self.enabled {
            self.series
                .entry(name.to_string())
                .or_default()
                .values
                .entry(label.to_string())
                .or_default()
                .push(value);
        }
    }

    /// Enable the collection of values.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Set the header for a given metric.
    /// Parameters:
    /// - `name`: the name of the metric.
    /// - `header`: the header to be used for serializing values.
    pub fn set_header(&mut self, name: &str, header: &str) {
        self.series.entry(name.to_string()).or_default().header = header.to_string();
    }
}

pub struct Output {
    pub single: OutputSingle,
    pub series: OutputSeries,
    pub config_csv: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_avg() -> anyhow::Result<()> {
        let warmups = [0, 5];
        let expected_values = [1.9, 2.0];
        for (warmup, expected_value) in warmups.iter().zip(expected_values.iter()) {
            let mut single = OutputSingle::new();
            single.enable(*warmup);
            single.time_avg("metric", 20, 1.0);
            single.time_avg("metric", 30, 2.0);
            single.time_avg("metric", 40, 1.0);
            single.time_avg("metric", 50, 3.0);
            single.finish(100);

            let metric = single.time_avg.get("metric").unwrap();

            assert!(
                metric.avg() == *expected_value,
                "{} != {} (sum {}, time {}, warmup {})",
                metric.avg(),
                *expected_value,
                metric.sum_values,
                metric.sum_time,
                warmup
            );
        }

        Ok(())
    }
}
