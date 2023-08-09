use anyhow::{bail, Context, Result};
use std::{fs, mem::MaybeUninit, os::fd::AsRawFd};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // get the first argument to the program
    let cid = match args.iter().nth(1) {
        Some(cid) => cid,
        None => {
            eprintln!("Usage: {} cid", args.first().unwrap());
            std::process::exit(1);
        }
    };

    // open the relevant file from ctfs
    let file = fs::File::open(format!("/system/contract/all/{cid}/status"))
        .context("failed to open contract status file")?;

    // safely create some storage for where the handle will be initialized
    let mut handle = MaybeUninit::<contract_sys::ct_stathdl_t>::uninit();

    // read the data into our handle
    unsafe {
        if contract_sys::ct_status_read(
            file.as_raw_fd(),
            contract_sys::CTD_ALL as i32,
            handle.as_mut_ptr(),
        ) != 0
        {
            bail!("failed to read contract status");
        };
    }

    // handle should be initialized now
    let handle = unsafe { handle.assume_init() };

    // we can close the file now
    drop(file);

    let mut numpids = 0;

    // create storage for a pointer to an array of pid_t
    let mut pids = MaybeUninit::<*mut libc::pid_t>::uninit();

    // get all of the members associated with the contract
    unsafe {
        if contract_sys::ct_pr_status_get_members(handle, pids.as_mut_ptr(), &mut numpids) != 0 {
            bail!("failed to get contract members");
        };
    }

    // provide a nice way for us to loop over the pids
    let pids = unsafe { std::slice::from_raw_parts(pids.assume_init(), numpids as usize) };
    println!("Here are the pids in contract {cid}:\n {pids:#?}");

    // cleanup the handle now that we are done
    unsafe {
        contract_sys::ct_status_free(handle);
    }

    Ok(())
}
