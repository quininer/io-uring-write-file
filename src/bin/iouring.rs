use io_uring::opcode::types::Fd;
use io_uring::opcode::Write;
use io_uring::squeue::Flags;
use io_uring::IoUring;
use io_uring_write_file::{Timer, DATA, TOTAL};
use libc::off_t;
use std::fs::File;
use std::io;
use std::os::unix::io::AsRawFd;

const BATCH_SIZE: usize = 32;

fn main() -> io::Result<()> {
    let file = File::create("iouring.text")?;

    let mut uring = IoUring::new(BATCH_SIZE as u32 * 2)?;

    let (submitter, sq, cq) = uring.split();
    let mut sq = sq.available();

    let timer = Timer::start();

    let mut submitted = 0;
    let mut completed = 0;

    while submitted != TOTAL {
        unsafe {
            let entry = Write::new(Fd(file.as_raw_fd()), DATA.as_ptr(), DATA.len() as u32)
                .offset(submitted as off_t)
                .build()
                .flags(Flags::ASYNC);

            match sq.push(entry) {
                Ok(()) => submitted += DATA.len(),
                Err(entry) => {
                    sq.sync();
                    completed += cq.available().count() * DATA.len();
                    while let Err(err) = submitter.submit() {
                        if err.raw_os_error() == Some(libc::EBUSY) {
                            completed += cq.available().count() * DATA.len();
                        } else {
                            return Err(err)
                        }
                    }
                    if sq.push(entry).is_ok() {
                        submitted += DATA.len();
                    }
                }
            }
        }
    }

    drop(sq);

    while completed != TOTAL {
        submitter.submit_and_wait(1)?;
        completed += cq.available().count() * DATA.len();
    }

    let elapsed = timer.stop();
    println!("{} ms", elapsed.as_millis());

    Ok(())
}
