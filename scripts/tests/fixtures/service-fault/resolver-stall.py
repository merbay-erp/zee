"""Test-only Python resolver dependency injection; never native DNS/network evidence."""
import signal,socket,sys,time
from pathlib import Path

def stalled(*args,**kwargs):
    Path(sys.argv[1]).write_text('resolver-entered')
    while True:time.sleep(.05)

# Only this fresh owned child is changed. No resolver configuration or global service changes.
signal.signal(signal.SIGTERM,signal.SIG_IGN)
socket.getaddrinfo=stalled
socket.getaddrinfo('anonymous.fixture.invalid',443)
