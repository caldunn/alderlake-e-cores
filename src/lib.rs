use async_process::Command;
use futures::future::try_join_all;
use std::env;
use std::ops::Range;
use std::process;
use std::process::Output;
use std::{fmt, io};

/// Potential Errors returned from public functions.
#[derive(Eq, PartialEq, Debug)]
pub enum CoreTypeFetchError {
    /// CPU contains a core that is not Intel Atom® or Intel® Core™
    UnknownCoreType,
    /// Taskset binary is missing or inaccessible.
    TasksetFailure,
    /// CPU is not a Hybrid CPU
    NotHybridCPU,
}

#[derive(Debug, Clone)]
struct NotHybridCPU;
impl fmt::Display for NotHybridCPU {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "This CPU does not have a hybrid architecture (eg. Alder Lake)"
        )
    }
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

/// Contains the range of indexes for each core type.
/// # Example
/// ```
/// use e_core_detection::CorePELayout;
/// let i7_1200k = CorePELayout {
///     p_cores: (0..15),
///     e_cores: (16..19),
/// };
/// ```
#[derive(Debug)]
pub struct CorePELayout {
    pub p_cores: Range<u8>,
    pub e_cores: Range<u8>,
}

impl CorePELayout {
    /// Create the p & e core ranges partitioned at the index of the last p-core.
    /// # Arguments
    /// * `p_core_end_idx`: The index of the last p core.
    /// returns: CorePELayout
    /// # Example
    /// ```
    /// use e_core_detection::CorePELayout;
    /// let i7_12700k = CorePELayout::new(15);
    /// ```
    pub fn new(p_core_end_idx: u8) -> CorePELayout {
        CorePELayout {
            p_cores: (0..p_core_end_idx),
            e_cores: (p_core_end_idx + 1..19),
        }
    }
    /// Simple format string. Used for output in binary.
    pub fn formatted_string(&self) -> String {
        format!("P CORES: {:?}\nE Cores: {:?}", self.p_cores, self.e_cores)
    }
}

/// Request the CorePELayout for the current system asynchronously.
pub async fn get_pe_partition_async() -> Result<CorePELayout, CoreTypeFetchError> {
    if !is_hybrid() {
        return Err(CoreTypeFetchError::NotHybridCPU);
    }

    taskset_async(num_cpus::get() as u8).await
}
/// Request the CorePELayout for the current system synchronously.
pub fn get_pe_partition_sync() -> Result<CorePELayout, CoreTypeFetchError> {
    if !is_hybrid() {
        return Err(CoreTypeFetchError::NotHybridCPU);
    }

    taskset_sync(num_cpus::get() as u8)
}

/// Determine whether the current core is a P or E core.
/// # Safety
/// Should be safe to call on a CPU that supports CPUID. Not sure on anything else.
pub unsafe fn test_core() -> String {
    let res = core::arch::x86_64::__cpuid_count(0x1A, 0x00);
    match res.eax >> 24 {
        64 => "P_CORE".to_string(),
        32 => "E_CORE".to_string(),
        x => format!(
            "UNRECOGNISED: indicator-range={:08b}, eax_reg={:032b}",
            x as u8, res.eax
        ),
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

fn process_raw_command_out(
    raw_out: Vec<io::Result<Output>>,
) -> Result<CorePELayout, UnknownCoreType> {
    let p_and_e: (Vec<_>, Vec<_>) = raw_out
        .into_iter()
        .map(|core_type| {
            match core_type {
                Ok(c) => {
                    let res = std::str::from_utf8(&*c.stdout)
                        .map_err(|_| CoreTypeFetchError::TasksetFailure)?;
                    CoreType::from_string(res.trim())
                }
                Err(_) => None,
            }
            .ok_or(CoreTypeFetchError::UnknownCoreType)
        })
        .partition(|ct| *ct == Ok(CoreType::P));

    Ok(CorePELayout::new(p_and_e.0.len() as u8 - 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _result = get_pe_partition_sync();
        assert_eq!(true, true);
    }
}
