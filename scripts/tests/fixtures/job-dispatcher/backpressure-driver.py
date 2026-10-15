import importlib.util,json,sys,time
from pathlib import Path
stage=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('host',stage/'job-dispatcher.py');host=importlib.util.module_from_spec(spec);spec.loader.exec_module(host)
binary,project,config,receipt=sys.argv[1:]
env=json.loads(Path(config).read_text());log=host.JsonLog();start=time.monotonic()
for _ in range(100000):log.emit('synthetic_pressure')
dropped_before=log.dropped
dispatcher=host.Dispatcher(Path(binary),Path(project),env,'full-log-pipe',log=log)
try:dispatcher.tick()
finally:dispatcher.cleanup()
Path(receipt).write_text(json.dumps({'dropped_before_sql':dropped_before,'dropped_after_sql':log.dropped,'applied':dispatcher.applied,'owned_children_remaining':len(dispatcher.children),'elapsed_seconds':time.monotonic()-start}))
