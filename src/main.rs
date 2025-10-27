#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate axlog;

extern crate alloc;
#[cfg(feature = "dyn")]
extern crate axdriver_dyn;
extern crate axruntime;

use alloc::{borrow::ToOwned, vec::Vec};

use axfs_ng::FS_CONTEXT;

mod entry;

pub const CMDLINE: &[&str] = &["/rknn_yolov8_demo/rknn_yolov8_demo", "/rknn_yolov8_demo/model/yolov8.rknn", "/rknn_yolov8_demo/model/bus.jpg"];
#[unsafe(no_mangle)]
fn main() {
    starry_api::init();

    let args = CMDLINE
        .iter()
        .copied()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let envs = [];
    let exit_code = entry::run_initproc(&args, &envs);
    info!("Init process exited with code: {:?}", exit_code);

    let cx = FS_CONTEXT.lock();
    cx.root_dir()
        .unmount_all()
        .expect("Failed to unmount all filesystems");
    cx.root_dir()
        .filesystem()
        .flush()
        .expect("Failed to flush rootfs");
}

#[cfg(feature = "vf2")]
extern crate axplat_riscv64_visionfive2;

#[cfg(feature = "2k1000la")]
extern crate axplat_loongarch64_2k1000la;

#[cfg(target_arch = "aarch64")]
extern crate axplat_aarch64_dyn;
