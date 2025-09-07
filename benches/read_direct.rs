use std::{path::PathBuf, sync::LazyLock};

use compio::io::AsyncReadManagedAt;
use divan::counter::BytesCount;

static EXE: LazyLock<PathBuf> = LazyLock::new(|| std::env::current_exe().unwrap());

fn main() {
    divan::main();
}

#[divan::bench(consts = [4096, 65536, 256 * 1024, 1024 * 1024], sample_size = 500)]
fn compio_read_exact_managed_at<const N: usize>(bencher: divan::Bencher) {
    let rt = compio::runtime::RuntimeBuilder::default().build().unwrap();

    let hndl = rt
        .block_on(
            compio::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_DIRECT)
                .open(EXE.as_path()),
        )
        .unwrap();

    let pool: compio::runtime::BufferPool = rt.block_on(async {
        <compio::fs::File as AsyncReadManagedAt>::BufferPool::new(4, N).unwrap()
    });

    bencher.counter(BytesCount::u8(N)).bench_local(move || {
        let mut left = N;
        rt.block_on(async {
            while left > 0 {
                let b = hndl.read_managed_at(&pool, N, (N - left) as u64).await.unwrap();
                left -= b.len();
            }
        });
    })
}