//! K-257 private native metadata-only counters and inherited nonblocking pipe.
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

pub(super) const MAX: u64 = 9_007_199_254_740_991;
const STATUS: [u16; 16] = [
    200, 303, 400, 401, 403, 404, 405, 408, 413, 426, 429, 431, 500, 503, 504, 0,
];
#[derive(Default)]
struct State {
    request: [u64; 22],
    db: [u64; 8],
    seq: u64,
    dropped: u64,
}
#[derive(serde::Serialize)]
struct Snapshot<'a> {
    schema: &'static str,
    kind: u8,
    seq: u64,
    dropped: u64,
    v: &'a [u64],
}
pub(super) struct Sink {
    state: Mutex<State>,
    writer: Mutex<Option<std::fs::File>>,
    active: AtomicBool,
    pipe_limit: usize,
}

fn add(n: &mut u64, delta: u64) {
    *n = n.saturating_add(delta).min(MAX);
}

#[derive(Debug, PartialEq)]
enum Decision {
    Complete,
    Dropped,
    Disable,
}
fn write_decision(result: std::io::Result<usize>, expected: usize) -> Decision {
    match result {
        Ok(n) if n == expected => Decision::Complete,
        Err(e)
            if matches!(
                e.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
            ) =>
        {
            Decision::Dropped
        }
        _ => Decision::Disable,
    }
}

impl Sink {
    #[cfg(unix)]
    fn inspect(fd: i32) -> Result<usize, String> {
        let rejected =
            || "native metrics requires an owned nonblocking write-only pipe".to_string();
        if !(3..=255).contains(&fd) {
            return Err(rejected());
        }
        // SAFETY: fstat/sigaction receive live aligned C POD outputs; querying
        // SIGPIPE with null new action does not change its disposition.
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        let mut signal: libc::sigaction = unsafe { std::mem::zeroed() };
        let valid = unsafe {
            libc::fstat(fd, &mut stat) == 0
                && libc::sigaction(libc::SIGPIPE, std::ptr::null(), &mut signal) == 0
        };
        // SAFETY: fcntl and fpathconf inspect the live inherited descriptor only.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        let pipe_limit = unsafe { libc::fpathconf(fd, libc::_PC_PIPE_BUF) };
        let mut stdio_alias = false;
        for standard in 0..=2 {
            // SAFETY: each fstat writes a live C POD output; failed stdio is ignored.
            let mut other: libc::stat = unsafe { std::mem::zeroed() };
            if unsafe { libc::fstat(standard, &mut other) } == 0
                && stat.st_dev == other.st_dev
                && stat.st_ino == other.st_ino
            {
                stdio_alias = true;
            }
        }
        if !valid || stdio_alias
            || stat.st_mode & libc::S_IFMT != libc::S_IFIFO
            // SAFETY: geteuid only reads the current process identity.
            || ![0, unsafe { libc::geteuid() }].contains(&stat.st_uid)
            || flags < 0
            || flags & libc::O_ACCMODE != libc::O_WRONLY
            || flags & libc::O_NONBLOCK == 0
            || pipe_limit < 512
            || signal.sa_sigaction != libc::SIG_IGN
        {
            return Err(rejected());
        }
        Ok((pipe_limit as usize).min(512))
    }
    pub(super) fn preflight(fd: i32) -> Result<(), String> {
        #[cfg(unix)]
        {
            Self::inspect(fd).map(|_| ())
        }
        #[cfg(not(unix))]
        {
            let _ = fd;
            Err("native metrics pipe requires Unix".into())
        }
    }
    #[cfg(unix)]
    pub(super) fn inherited(fd: i32) -> Result<Arc<Self>, String> {
        use std::os::fd::FromRawFd;
        let pipe_limit = Self::inspect(fd)?;
        // SAFETY: CLI exclusively transfers this verified inherited descriptor.
        // No pool/child thread exists yet; CLOEXEC prevents incidental forwarding.
        if unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) } != 0 {
            return Err("native metrics requires an owned nonblocking write-only pipe".into());
        }
        let writer = unsafe { std::fs::File::from_raw_fd(fd) };
        let sink = Arc::new(Self {
            state: Mutex::new(State::default()),
            writer: Mutex::new(Some(writer)),
            active: AtomicBool::new(true),
            pipe_limit,
        });
        sink.db([0; 8]);
        Ok(sink)
    }
    #[cfg(not(unix))]
    pub(super) fn inherited(_fd: i32) -> Result<Arc<Self>, String> {
        Err("native metrics pipe requires Unix".into())
    }

    #[cfg(test)]
    pub(super) fn counters_only_for_test() -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::default()),
            writer: Mutex::new(None),
            active: AtomicBool::new(false),
            pipe_limit: 512,
        })
    }
    #[cfg(test)]
    pub(super) fn request_test_state(&self) -> [u64; 22] {
        self.state.lock().unwrap().request
    }
    fn dropped(&self) {
        if let Ok(mut state) = self.state.lock() {
            add(&mut state.dropped, 1);
        }
    }
    fn change(&self, kind: u8, change: impl FnOnce(&mut State)) {
        let (v, seq, dropped) = {
            let Ok(mut state) = self.state.lock() else {
                self.active.store(false, Ordering::Relaxed);
                return;
            };
            change(&mut state);
            if !self.active.load(Ordering::Relaxed) {
                return;
            }
            if state.seq == MAX {
                self.active.store(false, Ordering::Relaxed);
                return;
            }
            state.seq += 1;
            let mut v = [0; 22];
            if kind == 0 {
                v = state.request;
            } else {
                v[..8].copy_from_slice(&state.db);
            }
            (v, state.seq, state.dropped)
        };
        let record = Snapshot {
            schema: "zee-native-metrics-1",
            kind,
            seq,
            dropped,
            v: if kind == 0 { &v } else { &v[..8] },
        };
        let Ok(mut bytes) = serde_json::to_vec(&record) else {
            self.dropped();
            return;
        };
        bytes.push(b'\n');
        if bytes.len() > self.pipe_limit {
            self.dropped();
            return;
        }
        // Formatting and the single write happen after the counter lock is dropped.
        // A writer contending with another snapshot never waits behind its IO.
        let Ok(mut writer) = self.writer.try_lock() else {
            self.dropped();
            return;
        };
        let Some(file) = writer.as_mut() else {
            return;
        };
        match write_decision(file.write(&bytes), bytes.len()) {
            Decision::Complete => {}
            Decision::Disable => {
                self.active.store(false, Ordering::Relaxed);
                *writer = None;
                self.dropped();
            }
            Decision::Dropped => self.dropped(),
        }
    }
    pub(super) fn accepted(&self) {
        self.change(0, |s| add(&mut s.request[0], 1));
    }
    fn complete(&self, elapsed: u64, outcome: Option<(u16, bool)>) {
        self.change(0, |s| {
            add(&mut s.request[1], 1);
            add(&mut s.request[2], elapsed);
            match outcome {
                Some((status, success)) => {
                    // Server-selected final attempted status; delivery is a
                    // separate write result, never inferred as client receipt.
                    let index = STATUS.iter().position(|n| *n == status).unwrap_or(15);
                    add(&mut s.request[6 + index], 1);
                    if !success {
                        add(&mut s.request[3], 1);
                    }
                }
                None => add(&mut s.request[4], 1),
            }
        });
    }
    pub(super) fn rate_reject(&self) {
        self.change(0, |s| add(&mut s.request[5], 1));
    }
    pub(super) fn pool_observer(self: &Arc<Self>) -> Arc<dyn Fn([u64; 8]) + Send + Sync> {
        let sink = Arc::clone(self);
        let last = Mutex::new([0u64; 4]);
        Arc::new(move |values| {
            let delta = {
                let Ok(mut last) = last.lock() else {
                    return;
                };
                let mut delta = [0; 4];
                for i in 0..4 {
                    delta[i] = values[i + 4].saturating_sub(last[i]);
                    last[i] = last[i].max(values[i + 4]);
                }
                delta
            };
            sink.change(1, |state| {
                state.db[..4].copy_from_slice(&values[..4]);
                for (target, delta) in state.db[4..].iter_mut().zip(delta) {
                    add(target, delta);
                }
            });
        })
    }
    #[cfg_attr(not(unix), allow(dead_code))]
    pub(super) fn db(&self, values: [u64; 8]) {
        self.change(1, |s| {
            for (dst, src) in s.db.iter_mut().zip(values) {
                *dst = src.min(MAX);
            }
        });
    }
}

pub(super) trait WebTasima: Read + Write {
    fn set_read_timeout(&self, duration: Option<std::time::Duration>) -> std::io::Result<()>;
    fn metrik_yanit(&mut self, _status: u16, _success: bool) {}
}
impl WebTasima for TcpStream {
    fn set_read_timeout(&self, duration: Option<std::time::Duration>) -> std::io::Result<()> {
        TcpStream::set_read_timeout(self, duration)
    }
}
pub(super) struct Akis {
    inner: TcpStream,
    sink: Option<Arc<Sink>>,
    start: Option<Instant>,
    outcome: Option<(u16, bool)>,
}
impl Akis {
    pub(super) fn new(inner: TcpStream, sink: Option<Arc<Sink>>) -> Self {
        let start = sink.as_ref().map(|_| Instant::now());
        if let Some(sink) = &sink {
            sink.accepted();
        }
        Self {
            inner,
            sink,
            start,
            outcome: None,
        }
    }
}
impl From<TcpStream> for Akis {
    fn from(stream: TcpStream) -> Self {
        Self::new(stream, None)
    }
}
impl std::ops::Deref for Akis {
    type Target = TcpStream;
    fn deref(&self) -> &TcpStream {
        &self.inner
    }
}
impl std::ops::DerefMut for Akis {
    fn deref_mut(&mut self) -> &mut TcpStream {
        &mut self.inner
    }
}
impl Read for Akis {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(bytes)
    }
}
impl Write for Akis {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.inner.write(bytes)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
impl WebTasima for Akis {
    fn set_read_timeout(&self, duration: Option<std::time::Duration>) -> std::io::Result<()> {
        self.inner.set_read_timeout(duration)
    }
    fn metrik_yanit(&mut self, status: u16, success: bool) {
        self.outcome = Some((status, success));
    }
}
impl Drop for Akis {
    fn drop(&mut self) {
        if let (Some(sink), Some(start)) = (&self.sink, self.start) {
            sink.complete(
                start.elapsed().as_micros().min(MAX as u128) as u64,
                self.outcome,
            );
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::fd::{AsRawFd, FromRawFd};
    fn pipe() -> (std::fs::File, i32) {
        let mut fds = [0; 2];
        // SAFETY: pipe writes two valid descriptors into live C integer storage.
        assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
        for fd in fds {
            assert_eq!(
                unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) },
                0
            );
        }
        // SAFETY: the returned read descriptor is transferred exactly once.
        (unsafe { std::fs::File::from_raw_fd(fds[0]) }, fds[1])
    }
    #[test]
    fn pool_replacements_keep_cumulative_totals_and_current_boundary_gauges() {
        let sink = Sink::counters_only_for_test();
        let first = sink.pool_observer();
        first([4, 0, 4, 1, 6, 1, 2000, 3000]);
        first([4, 4, 0, 0, 7, 1, 2010, 6000]);
        let second = sink.pool_observer();
        second([1, 0, 1, 0, 1, 0, 5, 0]);
        second([1, 1, 0, 0, 1, 0, 5, 10]);
        let values = sink.state.lock().unwrap().db;
        assert_eq!(&values[..4], &[1, 1, 0, 0]);
        assert_eq!(&values[4..], &[8, 1, 2015, 6010]);
    }
    #[test]
    fn cli_metric_descriptor_preserves_application_arguments() {
        let mut args = vec![
            "çalıştır".to_string(),
            "program.dil".into(),
            "--web-metrik-fd".into(),
            "5".into(),
        ];
        let before = args.clone();
        assert_eq!(super::super::metrik_fd_ayikla(&mut args).unwrap(), None);
        assert_eq!(args, before);
        let mut args = vec![
            "çalıştır".to_string(),
            "--web-proxy".into(),
            "https://fixture.example".into(),
            "--web-metrik-fd".into(),
            "5".into(),
            "program.dil".into(),
        ];
        assert_eq!(super::super::metrik_fd_ayikla(&mut args).unwrap(), Some(5));
        assert!(!args.contains(&"--web-metrik-fd".to_string()));
        let mut duplicate = vec![
            "çalıştır".to_string(),
            "--web-metrik-fd".into(),
            "5".into(),
            "--web-metrik-fd".into(),
            "6".into(),
            "program.dil".into(),
        ];
        assert!(super::super::metrik_fd_ayikla(&mut duplicate).is_err());
    }
    #[test]
    fn finite_maximum_snapshot_fits_posix_minimum_pipe_buf() {
        let values = [MAX; 22];
        let snapshot = Snapshot {
            schema: "zee-native-metrics-1",
            kind: 0,
            seq: MAX,
            dropped: MAX,
            v: &values,
        };
        let bytes = serde_json::to_vec(&snapshot).unwrap();
        assert!(bytes.len() < 512);
        assert!(!bytes.windows(3).any(|b| b == b"NaN"));
    }
    #[test]
    fn partial_zero_closed_and_interrupted_writes_have_fixed_decisions() {
        assert_eq!(write_decision(Ok(511), 512), Decision::Disable);
        assert_eq!(write_decision(Ok(0), 512), Decision::Disable);
        assert_eq!(write_decision(Ok(512), 512), Decision::Complete);
        assert_eq!(
            write_decision(Err(std::io::ErrorKind::BrokenPipe.into()), 512),
            Decision::Disable
        );
        assert_eq!(
            write_decision(Err(std::io::ErrorKind::WouldBlock.into()), 512),
            Decision::Dropped
        );
        assert_eq!(
            write_decision(Err(std::io::ErrorKind::Interrupted.into()), 512),
            Decision::Dropped
        );
    }
    #[test]
    fn full_real_pipe_drops_without_request_blocking() {
        let (_read, fd) = pipe();
        let sink = Sink::inherited(fd).unwrap();
        {
            let mut writer = sink.writer.lock().unwrap();
            let file = writer.as_mut().unwrap();
            loop {
                match file.write(&[b'x'; 512]) {
                    Ok(512) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    other => panic!("unexpected bounded fill {other:?}"),
                }
            }
            // The initial DB-zero snapshot can leave <512 bytes free. Fill
            // that remainder too so this fixture is genuinely at EAGAIN for
            // even the smaller request snapshot, rather than merely <PIPE_BUF.
            loop {
                match file.write(&[b'x'; 1]) {
                    Ok(1) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    other => panic!("unexpected byte fill {other:?}"),
                }
            }
        }
        let start = Instant::now();
        sink.accepted();
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
        assert!(sink.active.load(Ordering::Relaxed));
        assert_eq!(sink.state.lock().unwrap().dropped, 1);
    }
    #[test]
    fn closed_real_reader_disables_metrics_without_sigpipe() {
        let (read, fd) = pipe();
        let sink = Sink::inherited(fd).unwrap();
        drop(read);
        sink.accepted();
        assert!(!sink.active.load(Ordering::Relaxed));
        assert!(sink.writer.lock().unwrap().is_none());
    }
    #[test]
    fn wrong_descriptor_type_mode_and_blocking_pipe_rejected() {
        let file = std::fs::File::open("/dev/null").unwrap();
        assert!(Sink::inherited(file.as_raw_fd()).is_err());
        let (read, fd) = pipe();
        assert!(Sink::inherited(read.as_raw_fd()).is_err());
        assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFL, 0) }, 0);
        assert!(Sink::inherited(fd).is_err());
        // SAFETY: the rejected write descriptor remains owned by this fixture.
        unsafe { libc::close(fd) };
        assert!(Sink::inherited(-1).is_err());
        assert!(Sink::inherited(256).is_err());
    }
    #[test]
    fn observer_is_metadata_only_and_saturates() {
        let (read, fd) = pipe();
        let sink = Sink::inherited(fd).unwrap();
        sink.accepted();
        sink.rate_reject();
        sink.complete(1000, Some((429, true)));
        sink.accepted();
        sink.complete(MAX, Some((200, false)));
        let state = sink.state.lock().unwrap();
        assert_eq!(state.request[0], 2);
        assert_eq!(state.request[1], 2);
        assert_eq!(state.request[2], MAX);
        assert_eq!(state.request[3], 1);
        assert_eq!(state.request[5], 1);
        assert_eq!(state.request[16], 1);
        drop(state);
        drop(sink);
        let mut bytes = Vec::new();
        let mut read = read;
        read.read_to_end(&mut bytes).unwrap();
        for line in bytes.split(|b| *b == b'\n').filter(|b| !b.is_empty()) {
            let value: serde_json::Value = serde_json::from_slice(line).unwrap();
            assert_eq!(value.as_object().unwrap().len(), 5);
            assert_eq!(
                value["v"].as_array().unwrap().len(),
                if value["kind"] == 0 { 22 } else { 8 }
            );
        }
    }
}
