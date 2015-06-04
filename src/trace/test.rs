use super::*;

use std::io::Cursor;

fn test(data: &str, expected_frame_counts: &[usize]) {
    let mut frame_counts = vec![];
    let cursor = Cursor::new(data.as_bytes());
    each_trace(cursor, |frames| {
        frame_counts.push(frames.len());
    });
    assert_eq!(expected_frame_counts, &frame_counts[..]);
}

#[test]
fn test_run_1() {
    let data = r"
# ========
# captured on: Thu Jun  4 05:50:02 2015
# hostname : lunch-box
# os release : 3.13.0-52-generic
# perf version : 3.13.11-ckt18
# arch : x86_64
# nrcpus online : 16
# nrcpus avail : 16
# cpudesc : Intel(R) Xeon(R) CPU E5-2680 0 @ 2.70GHz
# cpuid : GenuineIntel,6,45,7
# total memory : 16355712 kB
# cmdline : /usr/lib/linux-tools-3.13.0-52/perf record -F 99 -g ./mach build -v -p script 
# event : name = cycles, type = 0, config = 0x0, config1 = 0x0, config2 = 0x0, excl_usr = 0, excl_kern = 0, excl_host = 0, excl_guest = 1, precise_ip = 0, attr_mmap2 = 0, attr_mmap  = 1, attr_mmap_data = 0
# HEADER_CPU_TOPOLOGY info available, use -I to display
# HEADER_NUMA_TOPOLOGY info available, use -I to display
# pmu mappings: cpu = 4, software = 1, uncore_pcu = 15, tracepoint = 2, uncore_imc_0 = 17, uncore_imc_1 = 18, uncore_imc_2 = 19, uncore_imc_3 = 20, uncore_qpi_0 = 21, uncore_qpi_1 = 22, uncore_cbox_0 = 7, uncore_cbox_1 = 8, uncore_cbox_2 = 9, uncore_cbox_3 = 10, uncore_cbox_4 = 11, uncore_cbox_5 = 12, uncore_cbox_6 = 13, uncore_cbox_7 = 14, uncore_ha = 16, uncore_r2pcie = 23, uncore_r3qpi_0 = 24, uncore_r3qpi_1 = 25, breakpoint = 5, uncore_ubox = 6
# ========
#
:18830 18830 2552105.017823: cycles: 
	ffffffff8104f45a [unknown] ([kernel.kallsyms])
	ffffffff8102f9ac [unknown] ([kernel.kallsyms])
	ffffffff81029c04 [unknown] ([kernel.kallsyms])
	ffffffff81142de7 [unknown] ([kernel.kallsyms])
	ffffffff81143f70 [unknown] ([kernel.kallsyms])
	ffffffff81146360 [unknown] ([kernel.kallsyms])
	ffffffff811c527f [unknown] ([kernel.kallsyms])
	ffffffff811c5c91 [unknown] ([kernel.kallsyms])
	ffffffff812156b8 [unknown] ([kernel.kallsyms])
	ffffffff811c43cf [unknown] ([kernel.kallsyms])
	ffffffff812139c5 [unknown] ([kernel.kallsyms])
	ffffffff811c43cf [unknown] ([kernel.kallsyms])
	ffffffff811c5947 [unknown] ([kernel.kallsyms])
	ffffffff811c5e16 [unknown] ([kernel.kallsyms])
	ffffffff817336a9 [unknown] ([kernel.kallsyms])
	    7f10408df337 [unknown] ([unknown])

:18830 18830 2552105.017830: cycles: 
	ffffffff8104f45a [unknown] ([kernel.kallsyms])
	ffffffff8102f9ac [unknown] ([kernel.kallsyms])
	ffffffff81029c04 [unknown] ([kernel.kallsyms])
	ffffffff81142de7 [unknown] ([kernel.kallsyms])
	ffffffff81143f70 [unknown] ([kernel.kallsyms])
	ffffffff81146360 [unknown] ([kernel.kallsyms])
	ffffffff811c527f [unknown] ([kernel.kallsyms])
	ffffffff811c5c91 [unknown] ([kernel.kallsyms])
	ffffffff812156b8 [unknown] ([kernel.kallsyms])
	ffffffff811c43cf [unknown] ([kernel.kallsyms])
	ffffffff812139c5 [unknown] ([kernel.kallsyms])
	ffffffff811c43cf [unknown] ([kernel.kallsyms])
	ffffffff811c5947 [unknown] ([kernel.kallsyms])
	ffffffff811c5e16 [unknown] ([kernel.kallsyms])
	ffffffff817336a9 [unknown] ([kernel.kallsyms])
	    7f10408df337 [unknown] ([unknown])";

    test(data, &[16, 16]);
}
