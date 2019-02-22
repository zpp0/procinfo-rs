//! Process scheduler statistics from `/proc/[pid]/schedstat`.

use std::fs::File;
use std::io::Result;

use libc::pid_t;
use nom::{space, line_ending};

use parsers::{map_result, parse_usize, read_to_end};

/// Process scheduling statistics.
#[derive(Debug, Default, PartialEq, Eq, Hash)]
pub struct Schedstat {
    /// Time spent on the cpu (in ns).
    pub sum_exec_runtime: usize,
    /// Time spent waiting on a runqueue (in ns).
    pub run_delay: usize,
    /// # of timeslices run on this cpu.
    pub pcount: usize,
}

/// Parses the schedstat file format.
named!(parse_schedstat<Schedstat>,
    chain!(sum_exec_runtime: parse_usize ~ space ~
           run_delay: parse_usize        ~ space ~
           pcount: parse_usize           ~ line_ending ~
           || { Schedstat { sum_exec_runtime: sum_exec_runtime,
                            run_delay: run_delay,
                            pcount: pcount } }));

/// Parses the provided schedstat file.
fn schedstat_file(file: &mut File) -> Result<Schedstat> {
    let mut buf = [0; 256];
    map_result(parse_schedstat(try!(read_to_end(file, &mut buf))))
}

/// Returns scheduler information for the process with the provided pid.
pub fn schedstat(pid: pid_t) -> Result<Schedstat> {
    schedstat_file(&mut try!(File::open(&format!("/proc/{}/schedstat", pid))))
}

/// Returns scheduler information for the current process.
pub fn schedstat_self() -> Result<Schedstat> {
    schedstat_file(&mut try!(File::open("/proc/self/schedstat")))
}

/// Returns scheduler information from the thread with the provided parent process ID and thread ID.
pub fn schedstat_task(process_id: pid_t, thread_id: pid_t) -> Result<Schedstat> {
    schedstat_file(&mut try!(File::open(&format!("/proc/{}/task/{}/schedstat", process_id, thread_id))))
}

#[cfg(test)]
mod tests {
    use parsers::tests::unwrap;
    use super::{parse_schedstat, schedstat, schedstat_self};

    /// Test that the system schedstat files can be parsed.
    #[test]
    fn test_schedstat() {
        schedstat_self().unwrap();
        schedstat(1).unwrap();
    }

    #[test]
    fn test_parse_schedstat() {
        let schedstat_text = b"297751702229 1936831953 8028005\n";
        let schedstat = unwrap(parse_schedstat(schedstat_text));
        assert_eq!(297751702229, schedstat.sum_exec_runtime);
        assert_eq!(1936831953, schedstat.run_delay);
        assert_eq!(8028005, schedstat.pcount);
    }
}

#[cfg(all(test, rustc_nightly))]
mod benches {
    extern crate test;

    use std::fs::File;

    use parsers::read_to_end;
    use super::{parse_schedstat, schedstat};

    #[bench]
    fn bench_schedstat(b: &mut test::Bencher) {
        b.iter(|| test::black_box(schedstat(1)));
    }

    #[bench]
    fn bench_schedstat_parse(b: &mut test::Bencher) {
        let mut buf = [0; 256];
        let schedstat = read_to_end(&mut File::open("/proc/1/schedstat").unwrap(), &mut buf).unwrap();
        b.iter(|| test::black_box(parse_schedstat(schedstat)));
    }
}
