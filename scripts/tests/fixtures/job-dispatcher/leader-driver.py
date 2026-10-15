import subprocess,sys,time
from pathlib import Path
pidfile=Path(sys.argv[1]);child=subprocess.Popen(sys.argv[2:]);pidfile.write_text(str(child.pid))
while True:time.sleep(1)
