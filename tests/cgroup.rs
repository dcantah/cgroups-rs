// Copyright (c) 2018 Levente Kurusa
// Copyright (c) 2020 And Group
//
// SPDX-License-Identifier: Apache-2.0 or MIT
//

//! Simple unit tests about the control groups system.
use cgroups_rs::memory::MemController;
use cgroups_rs::Controller;
use cgroups_rs::{Cgroup, CgroupPid, Subsystem};

#[test]
fn test_procs_iterator_cgroup() {
    let h = cgroups_rs::hierarchies::auto();
    let pid = libc::pid_t::from(nix::unistd::getpid()) as u64;
    let cg = Cgroup::new(h, String::from("test_procs_iterator_cgroup")).unwrap();
    {
        // Add a task to the control group.
        cg.add_task_by_tgid(CgroupPid::from(pid)).unwrap();

        let mut procs = cg.procs().into_iter();
        // Verify that the task is indeed in the xcontrol group
        assert_eq!(procs.next(), Some(CgroupPid::from(pid)));
        assert_eq!(procs.next(), None);

        // Now, try removing it.
        cg.remove_task_by_tgid(CgroupPid::from(pid)).unwrap();
        procs = cg.procs().into_iter();

        // Verify that it was indeed removed.
        assert_eq!(procs.next(), None);
    }
    cg.delete().unwrap();
}

#[test]
fn test_cgroup_with_relative_paths() {
    if cgroups_rs::hierarchies::is_cgroup2_unified_mode() {
        return;
    }
    let h = cgroups_rs::hierarchies::auto();
    let cgroup_root = h.root();
    let cgroup_name = "test_cgroup_with_relative_paths";

    let cg = Cgroup::load(h, String::from(cgroup_name));
    {
        let subsystems = cg.subsystems();
        subsystems.iter().for_each(|sub| match sub {
            Subsystem::Pid(c) => {
                let cgroup_path = c.path().to_str().unwrap();
                let relative_path = "/pids/";
                // cgroup_path = cgroup_root + relative_path + cgroup_name
                assert_eq!(
                    cgroup_path,
                    format!(
                        "{}{}{}",
                        cgroup_root.to_str().unwrap(),
                        relative_path,
                        cgroup_name
                    )
                );
            }
            Subsystem::Mem(c) => {
                let cgroup_path = c.path().to_str().unwrap();
                // cgroup_path = cgroup_root + relative_path + cgroup_name
                assert_eq!(
                    cgroup_path,
                    format!("{}/memory/{}", cgroup_root.to_str().unwrap(), cgroup_name)
                );
            }
            _ => {}
        });
    }
    cg.delete().unwrap();
}

#[test]
fn test_cgroup_v2() {
    if !cgroups_rs::hierarchies::is_cgroup2_unified_mode() {
        return;
    }
    let h = cgroups_rs::hierarchies::auto();
    let cg = Cgroup::new(h, String::from("test_v2")).unwrap();

    let mem_controller: &MemController = cg.controller_of().unwrap();
    let (mem, swp, rev) = (4 * 1024 * 1000, 2 * 1024 * 1000, 1024 * 1000);

    mem_controller.set_limit(mem).unwrap();
    mem_controller.set_memswap_limit(swp).unwrap();
    mem_controller.set_soft_limit(rev).unwrap();

    let memory_stat = mem_controller.memory_stat();
    println!("memory_stat {:?}", memory_stat);
    assert_eq!(mem, memory_stat.limit_in_bytes);
    assert_eq!(rev, memory_stat.soft_limit_in_bytes);

    let memswap = mem_controller.memswap();
    println!("memswap {:?}", memswap);
    assert_eq!(swp, memswap.limit_in_bytes);

    cg.delete().unwrap();
}

#[test]
fn test_tasks_iterator_cgroup_threaded_mode() {
    if !cgroups_rs::hierarchies::is_cgroup2_unified_mode() {
        return;
    }
    let h = cgroups_rs::hierarchies::auto();
    let pid = libc::pid_t::from(nix::unistd::getpid()) as u64;
    let cg = Cgroup::new(h, String::from("test_tasks_iterator_cgroup_threaded_mode")).unwrap();
    let h = cgroups_rs::hierarchies::auto();
    let specified_controllers = vec![String::from("cpuset"), String::from("cpu")];
    let cg_threaded = Cgroup::new_with_specified_controllers(
        h,
        String::from("test_tasks_iterator_cgroup_threaded_mode/threaded"),
        Some(specified_controllers),
    )
    .unwrap();
    cg_threaded.set_cgroup_type("threaded").unwrap();
    {
        // Add a task to the control group.
        cg.add_task_by_tgid(CgroupPid::from(pid)).unwrap();

        let mut procs = cg.procs().into_iter();
        // Verify that the task is indeed in the xcontrol group
        assert_eq!(procs.next(), Some(CgroupPid::from(pid)));
        assert_eq!(procs.next(), None);

        // Add a task to the sub control group.
        cg_threaded.add_task(CgroupPid::from(pid)).unwrap();

        let mut tasks = cg_threaded.tasks().into_iter();
        // Verify that the task is indeed in the xcontrol group
        assert_eq!(tasks.next(), Some(CgroupPid::from(pid)));
        assert_eq!(tasks.next(), None);

        // Now, try move it to parent.
        cg_threaded
            .move_task_to_parent(CgroupPid::from(pid))
            .unwrap();
        tasks = cg_threaded.tasks().into_iter();

        // Verify that it was indeed removed.
        assert_eq!(tasks.next(), None);

        // Now, try removing it.
        cg.remove_task_by_tgid(CgroupPid::from(pid)).unwrap();
        procs = cg.procs().into_iter();

        // Verify that it was indeed removed.
        assert_eq!(procs.next(), None);
    }
    cg_threaded.delete().unwrap();
    cg.delete().unwrap();
}
