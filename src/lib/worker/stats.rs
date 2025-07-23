use serde::{ser::SerializeStruct, Serialize};
use sysinfo::{Disks, System};
use crate::lib::worker::types::{SystemStats};

impl Serialize for SystemStats {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SystemStats", 8)?;
        state.serialize_field("cpu_usage", &format!("{:.2}%", self.cpu_usage))?;
        state.serialize_field("total_memory", &format!("{} MB", self.total_memory))?;
        state.serialize_field("used_memory", &format!("{} MB", self.used_memory))?;
        state.serialize_field("total_swap", &format!("{} MB", self.total_swap))?;
        state.serialize_field("used_swap", &format!("{} MB", self.used_swap))?;
        state.serialize_field("system_name", &self.system_name)?;
        state.serialize_field("hostname", &self.hostname)?;
        state.serialize_field("total_cpus", &self.total_cpus)?;
        state.serialize_field("disk_usage", &format!("{:.2}%", self.disk_usage))?;
        state.serialize_field("task_count", &self.task_count)?;
        state.end()
    }
}

pub fn get_stats(sysinfo: &System, task_count: u64) -> SystemStats {
    SystemStats {
        cpu_usage: (sysinfo.global_cpu_usage() * 100.0).round() / 100.0,
        total_memory: sysinfo.total_memory() / 1024 / 1024,
        used_memory: sysinfo.used_memory() / 1024 / 1024,
        total_swap: sysinfo.total_swap() / 1024 / 1024,
        used_swap: sysinfo.used_swap() / 1024 / 1024,
        system_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        total_cpus: sysinfo.cpus().len() as u64,
        disk_usage: {
            let disks = Disks::new_with_refreshed_list();
            let used_space: f32 = disks
                .iter()
                .map(|disk| disk.total_space() as f32 - disk.available_space() as f32)
                .sum();
            let total_space: f32 = disks.iter().map(|disk| disk.total_space() as f32).sum();
            if total_space > 0.0 {
                (used_space / total_space) * 100.0
            } else {
                0.0
            }
        },
        task_count,
    }
}
