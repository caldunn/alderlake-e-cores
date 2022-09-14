use async_process::Command;
use futures::future::try_join_all;
use std::ops::Range;
use std::process;
use std::process::Output;
use std::{fmt, io};
use std::env;

#[derive(Debug, Clone)]
pub struct NotHybridCPU;
impl fmt::Display for NotHybridCPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "This CPU does not have a hybrid architecture (eg. Alder Lake)"
        )
    }
}

#[derive(PartialEq, Debug)]
pub enum CoreTypeFetchError {
    UnknownCoreType,
    TasksetFailure,
    NotHybridCPU,
}

#[derive(Debug, Clone)]
struct UnknownCoreType;
impl fmt::Display for UnknownCoreType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error reading hybrid core type")
    }
}

#[derive(Debug, Clone)]
struct TasksetFailure;
impl fmt::Display for TasksetFailure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Running on taskset failed")
    }
}

#[derive(Debug)]
pub struct CorePELayout {
    p_cores: Range<u8>,
    e_cores: Range<u8>,
}

impl CorePELayout {
    pub fn new(p_core_end_idx: u8) -> CorePELayout {
        CorePELayout {
            p_cores: (0..p_core_end_idx),
            e_cores: (p_core_end_idx + 1..19),
        }
    }
    pub fn formatted_string(&self) -> String {
        format!("P CORES: {:?}\nE Cores: {:?}", self.p_cores, self.e_cores)
    }
}

#[derive(PartialEq, Debug)]
enum CoreType {
    P,
    E,
}

impl CoreType {
    fn from_string(input: &str) -> Option<CoreType> {
        match input {
            "P_CORE" => Some(CoreType::P),
            "E_CORE" => Some(CoreType::E),
            _ => None,
        }
    }
}

fn is_hybrid() -> bool {
    unsafe {
        let res = core::arch::x86_64::__cpuid_count(0x07, 0x00);
        (1 & (res.edx >> (15 - 1))) == 1
    }
}

pub async fn get_pe_partition_async() -> Result<CorePELayout, CoreTypeFetchError> {
    if !is_hybrid() {
        return Err(CoreTypeFetchError::NotHybridCPU);
    }

    taskset_async(num_cpus::get() as u8).await
}

pub fn get_pe_partition_sync() -> Result<CorePELayout, CoreTypeFetchError> {
    if !is_hybrid() {
        return Err(CoreTypeFetchError::NotHybridCPU);
    }

    taskset_sync(num_cpus::get() as u8)
}

async fn taskset_async(n_cores: u8) -> Result<CorePELayout, CoreTypeFetchError> {
    let promises: Vec<_> = (0..n_cores)
        .into_iter()
        .map(|i| {
            tokio::spawn({
                Command::new("taskset")
                    .arg("--cpu-list")
                    .arg(i.to_string())
                    .arg(env::args().next().unwrap())
                    .arg("-s")
                    .output()
            })
        })
        .collect();

    let completed = try_join_all(promises)
        .await
        .map_err(|_| CoreTypeFetchError::TasksetFailure)?;

    process_raw_command_out(completed).map_err(|_| CoreTypeFetchError::UnknownCoreType)
}

fn taskset_sync(n_cores: u8) -> Result<CorePELayout, CoreTypeFetchError> {
    let cores_raw_strs: Vec<_> = (0..n_cores)
        .map(|i| {
            process::Command::new("taskset")
                .arg("--cpu-list")
                .arg(i.to_string())
                .arg(env::args().next().unwrap())
                .arg("-s")
                .output()
        })
        .collect();

    process_raw_command_out(cores_raw_strs).map_err(|_| CoreTypeFetchError::UnknownCoreType)
}

/// This will need to be re-written to handle errors cleaner.
///
/// # Arguments
///
/// * `raw_out`:
///
/// returns: Result<CorePELayout, UnknownCoreType>
///
/// # Examples
///
/// ```
///
/// ```
fn process_raw_command_out(
    raw_out: Vec<io::Result<Output>>,
) -> Result<CorePELayout, UnknownCoreType> {
    let as_enum: Vec<_> = raw_out
        .into_iter()
        .map(|core_type| {
            match core_type {
                Ok(c) => {
                    let res = std::str::from_utf8(&*c.stdout).map_err(|_|
                        CoreTypeFetchError::TasksetFailure)?;
                    CoreType::from_string(res.trim())
                }
                Err(_) => None,
            }
            .ok_or(CoreTypeFetchError::UnknownCoreType)
        })
        .collect();

    let p_and_e: (Vec<_>, Vec<_>) = as_enum.into_iter().partition(|ct| *ct == Ok(CoreType::P));

    Ok(CorePELayout::new(p_and_e.0.len() as u8 - 1))
}

pub unsafe fn test_core() {
    let res = core::arch::x86_64::__cpuid_count(0x1A, 0x00);
    match res.eax >> 24 {
        64 => println!("P_CORE"),
        32 => println!("E_CORE"),
        x => println!("UNRECOGNISED: indicator-range={}, eax_reg={}",
                      format!("{:08b}", x as u8),
                      format!("{:032b}", res.eax))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _result = get_pe_partition_sync().unwrap();
        assert_eq!(true, true);
    }
}
