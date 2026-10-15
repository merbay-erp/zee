//! Owned Unix listener handoff candidate. Private CLI adapter, no facade export.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Profile {
    pub listener: i32,
    pub ready: i32,
    pub activate: i32,
    pub generation: String,
    pub metric: Option<i32>,
    pub port: u16,
}
impl Profile {
    pub fn extract(args: &mut Vec<String>) -> Result<Option<Self>, String> {
        let flags = [
            "--web-listener-fd",
            "--web-ready-fd",
            "--web-activate-fd",
            "--web-generation",
        ];
        let mut values: [Option<String>; 4] = [None, None, None, None];
        let mut metric: Option<String> = None;
        let mut metric_repeated = false;
        let mut port = None;
        let mut i = 1;
        while i < args.len() {
            if let Some(n) = flags.iter().position(|flag| args[i] == *flag) {
                if values[n].is_some() {
                    return Err("native listener option repeated".into());
                }
                let value = args
                    .get(i + 1)
                    .ok_or("native listener option needs value")?
                    .clone();
                values[n] = Some(value);
                args.remove(i + 1);
                args.remove(i);
                continue;
            }
            if args[i] == "--web-worker-port" {
                port = args.get(i + 1).cloned();
                i += 2;
                continue;
            }
            if args[i] == "--web-metrik-fd" {
                metric_repeated |= metric.is_some();
                metric = args.get(i + 1).cloned();
                i += 2;
                continue;
            }
            if !args[i].starts_with('-') {
                break;
            }
            i += if option_has_value(&args[i]) { 2 } else { 1 };
        }
        if values.iter().all(Option::is_none) {
            return Ok(None);
        }
        if metric_repeated {
            return Err("metric descriptor repeated".into());
        }
        if cfg!(not(unix)) {
            return Err("native listener needs Unix".into());
        }
        let fd = |n: usize| -> Result<i32, String> {
            let value = values[n]
                .as_ref()
                .ok_or("native listener profile incomplete")?
                .parse::<i32>()
                .map_err(|_| "native listener invalid descriptor")?;
            if value < 3 {
                return Err("native listener descriptor must exceed stderr".into());
            }
            Ok(value)
        };
        let p = Self {
            listener: fd(0)?,
            ready: fd(1)?,
            activate: fd(2)?,
            port: port
                .ok_or("native listener needs explicit port")?
                .parse::<u16>()
                .map_err(|_| "native listener invalid port")?,
            metric: metric
                .map(|value| value.parse::<i32>())
                .transpose()
                .map_err(|_| "metric descriptor invalid")?,
            generation: values[3]
                .clone()
                .ok_or("native listener profile incomplete")?,
        };
        if p.listener == p.ready
            || p.listener == p.activate
            || p.ready == p.activate
            || p.generation.len() != 32
            || !p
                .generation
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("native listener collision or nonce".into());
        }
        Ok(Some(p))
    }
}
// Shared value walker: quiet detection does not grant an option/profile.
fn option_has_value(option: &str) -> bool {
    matches!(
        option,
        "--web-proxy"
            | "--güvenli-web-proxy"
            | "--web-worker-port"
            | "--web-oran-deposu"
            | "--web-oran-siniri"
            | "--web-oran-kapasitesi"
            | "--web-oran-doluluk"
            | "--ayar"
            | "--profil"
            | "--kapsam"
            | "--web-suite"
            | "--web-suite-sha256"
            | "--web-fixtures"
            | "--test-python"
            | "--test-postgres-bin"
            | "--test-output"
            | "--web-listener-fd"
            | "--web-ready-fd"
            | "--web-activate-fd"
            | "--web-generation"
            | "--web-metrik-fd"
    )
}

#[cfg(unix)]
mod unix {
    use super::Profile;
    use std::{
        fs::File,
        io::{Read, Write},
        mem,
        net::TcpListener,
        os::fd::FromRawFd,
        time::{Duration, Instant},
    };
    pub(crate) struct Control {
        activate: Option<File>,
        ready: Option<File>,
        port: u16,
        generation: String,
    }
    fn fail() -> String {
        "native listener descriptor or control rejected".into()
    }
    fn option(fd: i32, level: i32, name: i32) -> Result<i32, String> {
        let mut value = 0i32;
        let mut size = mem::size_of::<i32>() as libc::socklen_t;
        // SAFETY: points to correctly sized writable integer and length, borrowed fd only.
        if unsafe { libc::getsockopt(fd, level, name, (&mut value as *mut i32).cast(), &mut size) }
            != 0
            || size as usize != mem::size_of::<i32>()
        {
            return Err(fail());
        }
        Ok(value)
    }
    fn mode(fd: i32) -> Result<i32, String> {
        // SAFETY: fcntl reads flags from borrowed descriptor without acquiring ownership.
        let value = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if value < 0 {
            return Err(fail());
        }
        Ok(value)
    }
    fn pipe(fd: i32, access: i32) -> Result<(), String> {
        // SAFETY: zeroed stat is initialized by fstat before any field is read.
        let mut s: libc::stat = unsafe { mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut s) } != 0
            || s.st_mode & libc::S_IFMT != libc::S_IFIFO
            || mode(fd)? & libc::O_ACCMODE != access
        {
            return Err(fail());
        }
        Ok(())
    }
    fn listening(fd: i32) -> Result<bool, String> {
        #[cfg(target_vendor = "apple")]
        {
            // Darwin SO_ACCEPTCONN query returns ENOPROTOOPT; SDK tcp_fsm.h
            // defines TCPS_LISTEN=1. TCP_CONNECTION_INFO also proves TCP protocol.
            let mut info: libc::tcp_connection_info = unsafe { mem::zeroed() };
            let mut size = mem::size_of_val(&info) as libc::socklen_t;
            // SAFETY: borrowed fd, correctly sized writable native SDK record.
            if unsafe {
                libc::getsockopt(
                    fd,
                    libc::IPPROTO_TCP,
                    libc::TCP_CONNECTION_INFO,
                    (&mut info as *mut libc::tcp_connection_info).cast(),
                    &mut size,
                )
            } != 0
                || size == 0
                || size as usize > mem::size_of_val(&info)
            {
                return Err(fail());
            }
            Ok(info.tcpi_state == 1)
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            Ok(option(fd, libc::SOL_SOCKET, libc::SO_ACCEPTCONN)? == 1)
        }
    }
    fn identity(fd: i32) -> Result<(libc::dev_t, libc::ino_t), String> {
        // SAFETY: fstat initializes native stat; borrowed handle only.
        let mut value: libc::stat = unsafe { mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut value) } != 0 {
            return Err(fail());
        }
        Ok((value.st_dev, value.st_ino))
    }
    #[cfg(target_vendor = "apple")]
    #[repr(C)]
    struct VInfoStat {
        dev: u32,
        mode: u16,
        nlink: u16,
        ino: u64,
        uid: u32,
        gid: u32,
        times: [i64; 8],
        size: i64,
        blocks: i64,
        blksize: i32,
        flags: u32,
        generation: u32,
        rdev: u32,
        spare: [i64; 2],
    }
    #[cfg(target_vendor = "apple")]
    #[repr(C)]
    struct ProcPipeInfo {
        openflags: u32,
        status: u32,
        offset: i64,
        kind: i32,
        guardflags: u32,
        stat: VInfoStat,
        handle: u64,
        peer: u64,
        pipe_status: i32,
        reserved: i32,
    }
    fn pipe_pair(fd: i32, allow_closed_peer: bool) -> Result<Option<(u64, u64)>, String> {
        #[cfg(target_vendor = "apple")]
        {
            let mut stat: libc::stat = unsafe { mem::zeroed() };
            if unsafe { libc::fstat(fd, &mut stat) } != 0 {
                return Ok(None);
            }
            if stat.st_mode & libc::S_IFMT != libc::S_IFIFO {
                return Ok(None);
            }
            // sys/proc_info.h PROC_PIDFDPIPEINFO=6, exact native pipe_fdinfo ABI.
            // Own PID only; peer handles are compared and never logged/adopted.
            let mut info: ProcPipeInfo = unsafe { mem::zeroed() };
            let size = mem::size_of_val(&info) as i32;
            // SAFETY: own process, borrowed fd and exact writable SDK ABI buffer.
            if unsafe {
                libc::proc_pidfdinfo(
                    std::process::id() as i32,
                    fd,
                    6,
                    (&mut info as *mut ProcPipeInfo).cast(),
                    size,
                )
            } != size
                || info.handle == 0
                || (info.peer == 0 && !allow_closed_peer)
            {
                return Err(fail());
            }
            let peer = if info.peer == 0 {
                info.handle
            } else {
                info.peer
            };
            Ok(Some((info.handle.min(peer), info.handle.max(peer))))
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            let _ = (fd, allow_closed_peer);
            Ok(None)
        }
    }
    fn endpoint(fd: i32, port: u16) -> Result<(), String> {
        if port == 0 {
            return Err(fail());
        }
        // SAFETY: sockaddr storage is initialized by getsockname; exact length/family checked.
        let mut address: libc::sockaddr_in = unsafe { mem::zeroed() };
        let mut size = mem::size_of_val(&address) as libc::socklen_t;
        if unsafe {
            libc::getsockname(
                fd,
                (&mut address as *mut libc::sockaddr_in).cast(),
                &mut size,
            )
        } != 0
            || size as usize != mem::size_of_val(&address)
            || address.sin_family as i32 != libc::AF_INET
            || u16::from_be(address.sin_port) != port
            || address.sin_addr.s_addr.to_ne_bytes() != [127, 0, 0, 1]
        {
            return Err(fail());
        }
        Ok(())
    }
    pub(crate) fn aliases_stdio(fd: i32) -> bool {
        if (0..=2).contains(&fd) {
            return true;
        }
        let Ok(value) = identity(fd) else {
            return false;
        };
        let pair = pipe_pair(fd, true).ok().flatten();
        (0..=2).any(|stdio| {
            identity(stdio).is_ok_and(|other| other == value)
                || pair.is_some_and(|pair| {
                    pipe_pair(stdio, true).is_ok_and(|other| other == Some(pair))
                })
        })
    }
    pub(crate) fn preflight(p: &Profile) -> Result<(), String> {
        let mut roles = vec![p.listener, p.ready, p.activate];
        if let Some(metric) = p.metric {
            roles.push(metric);
        }
        let mut seen = Vec::new();
        let mut pipe_seen = Vec::new();
        for fd in roles {
            if fd < 3 {
                return Err(fail());
            }
            let value = identity(fd)?;
            if seen.contains(&value)
                || (0..=2).any(|stdio| identity(stdio).is_ok_and(|s| s == value))
            {
                return Err(fail());
            }
            // An optional metric reader may already be closed: the writer
            // remains valid and its sink must disable on EPIPE without stopping
            // the listener. Both control roles still require live peers.
            if let Some(pair) = pipe_pair(fd, p.metric == Some(fd))? {
                if pipe_seen.contains(&pair)
                    || (0..=2).any(|stdio| pipe_pair(stdio, true).is_ok_and(|p| p == Some(pair)))
                {
                    return Err(fail());
                }
                pipe_seen.push(pair);
            }
            seen.push(value);
        }
        pipe(p.ready, libc::O_WRONLY)?;
        pipe(p.activate, libc::O_RDONLY)?;
        if option(p.listener, libc::SOL_SOCKET, libc::SO_TYPE)? != libc::SOCK_STREAM
            || !listening(p.listener)?
            || mode(p.listener)? & libc::O_NONBLOCK == 0
        {
            return Err(fail());
        }
        endpoint(p.listener, p.port)?;
        Ok(())
    }
    pub(crate) fn install(p: &Profile, port: u16) -> Result<(TcpListener, Control), String> {
        preflight(p)?;
        if option(p.listener, libc::SOL_SOCKET, libc::SO_TYPE)? != libc::SOCK_STREAM
            || !listening(p.listener)?
            || mode(p.listener)? & libc::O_NONBLOCK == 0
        {
            return Err(fail());
        }
        // A TCP-specific option rejects other AF_INET stream protocols; does not mutate it.
        option(p.listener, libc::IPPROTO_TCP, libc::TCP_NODELAY)?;
        endpoint(p.listener, port)?;
        pipe(p.ready, libc::O_WRONLY)?;
        pipe(p.activate, libc::O_RDONLY)?;
        for fd in [p.listener, p.ready, p.activate] {
            // SAFETY: flags mutation on fully validated exact child-owned descriptors.
            let previous = unsafe { libc::fcntl(fd, libc::F_GETFD) };
            if previous < 0
                || unsafe { libc::fcntl(fd, libc::F_SETFD, previous | libc::FD_CLOEXEC) } != 0
            {
                return Err(fail());
            }
        }
        // Ready write must not block on an unread parent pipe.
        let ready_flags = mode(p.ready)?;
        if unsafe { libc::fcntl(p.ready, libc::F_SETFL, ready_flags | libc::O_NONBLOCK) } != 0 {
            return Err(fail());
        }
        // Poll readiness can race with another reader sharing the description.
        // Never let a blocking pipe read outlive the activation deadline.
        let activate_flags = mode(p.activate)?;
        if unsafe { libc::fcntl(p.activate, libc::F_SETFL, activate_flags | libc::O_NONBLOCK) } != 0
        {
            return Err(fail());
        }
        // SAFETY: all validation completed; distinct raw descriptors transfer exactly once.
        let listener = unsafe { TcpListener::from_raw_fd(p.listener) };
        let ready = unsafe { File::from_raw_fd(p.ready) };
        let activate = unsafe { File::from_raw_fd(p.activate) };
        Ok((
            listener,
            Control {
                activate: Some(activate),
                ready: Some(ready),
                port,
                generation: p.generation.clone(),
            },
        ))
    }
    impl Control {
        pub(crate) fn announce(&mut self) -> Result<(), String> {
            let mut ready = self.ready.take().ok_or_else(fail)?;
            let build: serde_json::Value =
                serde_json::from_str(dil::surum::json()).map_err(|_| fail())?;
            let data = format!(
                "{}\n",
                serde_json::json!({"schema":"zee-native-listener-ready-1","pid":std::process::id(),"generation":self.generation,"port":self.port,"build":build})
            );
            if data.len() > 4096 {
                return Err(fail());
            }
            ready.write_all(data.as_bytes()).map_err(|_| fail())
        }
        pub(crate) fn wait(&mut self, stopped: impl Fn() -> bool) -> Result<(), String> {
            let Some(mut file) = self.activate.take() else {
                return Ok(());
            };
            use std::os::fd::AsRawFd;
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut bytes = Vec::new();
            loop {
                if stopped() || Instant::now() >= deadline {
                    return Err(fail());
                }
                let mut poll = libc::pollfd {
                    fd: file.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                // SAFETY: single initialized pollfd, valid owned pipe; bounded timeout.
                let remaining = deadline.saturating_duration_since(Instant::now());
                let timeout_ms = remaining.as_millis().saturating_add(1).min(100) as i32;
                let result = unsafe { libc::poll(&mut poll, 1, timeout_ms) };
                if result < 0 {
                    if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    return Err(fail());
                }
                if result == 0 {
                    continue;
                }
                let mut chunk = [0u8; 64];
                let count = match file.read(&mut chunk) {
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) =>
                    {
                        continue
                    }
                    result => result.map_err(|_| fail())?,
                };
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..count]);
                if bytes.len() > 64 {
                    return Err(fail());
                }
            }
            if bytes != format!("ACTIVATE {}\n", self.generation).as_bytes() {
                return Err(fail());
            }
            if stopped() || Instant::now() >= deadline {
                return Err(fail());
            }
            Ok(())
        }
    }
}
#[cfg(unix)]
pub(crate) use unix::{install, preflight, Control};

pub(crate) fn quiet_preflight(args: &[String]) -> Result<(), String> {
    #[cfg(unix)]
    {
        // Borrow only; inspect every lateral FD before any parse/config diagnostic.
        // '=' is recognized only to keep rejected malformed profiles quiet.
        let roles = [
            "--web-listener-fd",
            "--web-ready-fd",
            "--web-activate-fd",
            "--web-metrik-fd",
        ];
        let mut i = 1;
        while let Some(arg) = args.get(i) {
            if !arg.starts_with('-') {
                break;
            }
            let (name, inline) = arg
                .split_once('=')
                .map_or((arg.as_str(), None), |(n, v)| (n, Some(v)));
            if roles.contains(&name) {
                if let Some(fd) = inline
                    .or_else(|| args.get(i + 1).map(String::as_str))
                    .and_then(|s| s.parse::<i32>().ok())
                    .filter(|fd| *fd >= 0)
                {
                    if unix::aliases_stdio(fd) {
                        return Err("native control stdio alias rejected".into());
                    }
                }
            }
            i += if inline.is_none()
                && option_has_value(name)
                && args.get(i + 1).is_some_and(|v| !v.starts_with('-'))
            {
                2
            } else {
                1
            };
        }
    }
    let mut borrowed = args.to_vec();
    if let Some(profile) = Profile::extract(&mut borrowed)? {
        #[cfg(unix)]
        preflight(&profile)?;
        #[cfg(not(unix))]
        return Err("native listener needs Unix".into());
    }
    Ok(())
}
