#!/usr/bin/env python3
"""K260 explicit metadata capability: actual subprocess descriptors, finite content."""
import fcntl,json,os,runpy,signal,subprocess,sys,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
TOOL=ROOT/'scripts/credential-baslat.py'
API=runpy.run_path(str(TOOL))
class Capability(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='zee-k260-');self.root=Path(self.temp.name).resolve();self.creds=self.root/'credentials';self.creds.mkdir(mode=0o700);self.key=self.creds/'key';self.key.write_text('SYNTHETIC_K260_CREDENTIAL_CANARY');self.key.chmod(0o400)
        self.source=self.root/'source.dil';self.source.write_text('SYNTHETIC_K260_SOURCE_CANARY !!! malformed\n');self.marker=self.root/'child.called'
        self.child=self.root/'native-fixture';self.child.write_text('#!'+sys.executable+'\nimport os,json,sys\nfrom pathlib import Path\nPath('+repr(str(self.marker))+').write_text("called")\nfd=int(sys.argv[sys.argv.index("--web-metrik-fd")+1])\nassert os.environ["ZEE_K260_KEY"]=="SYNTHETIC_K260_CREDENTIAL_CANARY"\ntry: os.fstat(int(os.environ["ZEE_K260_AMBIENT"])); ambient=True\nexcept OSError: ambient=False\nassert not ambient\nos.write(fd,json.dumps({"schema":"zee-k260-capability-proof-1","credential_consumed":True,"ambient_absent":True}).encode()+b"\\n")\n')
        self.child.chmod(0o700);self.fds=[]
    def tearDown(self):
        for fd in self.fds:
            try:os.close(fd)
            except OSError:pass
        self.temp.cleanup()
    def pipe(self):
        rd,wr=os.pipe();os.set_blocking(rd,False);os.set_blocking(wr,False);self.fds.extend((rd,wr));return rd,wr
    def command(self,fd,bridge=True):
        argv=[sys.executable,'-I','-B',str(TOOL),'--credential-root',str(self.creds),'--env','ZEE_K260_KEY=key','--dil',str(self.child)]
        if bridge:argv+=['--native-metrics-fd',str(fd)]
        return argv+['--','çalıştır','--web-proxy','https://fixture.example','--web-metrik-fd',str(fd),str(self.source)]
    def launch(self,argv,passed=(),**kw):
        env=os.environ.copy();env.pop('ZEE_K260_KEY',None);env['ZEE_K260_AMBIENT']=str(self.ambient)
        return subprocess.run(argv,env=env,stdin=subprocess.DEVNULL,stdout=kw.pop('stdout',subprocess.PIPE),stderr=kw.pop('stderr',subprocess.PIPE),pass_fds=passed,timeout=5,**kw)
    def ambient_pipe(self):
        rd,self.ambient=self.pipe();os.set_inheritable(self.ambient,True)
    def rejected(self,result,typed=True):
        self.assertEqual(result.returncode,2);self.assertFalse(self.marker.exists());metadata=json.loads(result.stdout);self.assertEqual(metadata['source'],'launcher');self.assertEqual(metadata['exit'],2)
        if typed:self.assertEqual(metadata['reason'],'native_metrics_capability')
        self.assertNotIn(b'CANARY',(result.stdout or b'')+(result.stderr or b''))
    def test_designated_fd_actual_child_only_and_default_noinherit(self):
        self.ambient_pipe();rd,wr=self.pipe();r=self.launch(self.command(wr),passed=(wr,self.ambient));self.assertEqual(r.returncode,0);self.assertEqual(json.loads(r.stdout),{'schema':'zee-credential-baslat-1','source':'child','exit':0});raw=os.read(rd,512);self.assertEqual(json.loads(raw),{'schema':'zee-k260-capability-proof-1','credential_consumed':True,'ambient_absent':True});self.assertNotIn(b'CANARY',raw+r.stdout+r.stderr)
    def test_absent_capability_never_implicitly_inherits_metrics_fd(self):
        self.ambient_pipe();rd,wr=self.pipe();r=self.launch(self.command(wr,False),passed=(wr,self.ambient));self.assertNotEqual(r.returncode,0)
        with self.assertRaises(BlockingIOError):os.read(rd,512)
    def test_closed_negative_and_wrong_mode_and_file(self):
        self.ambient_pipe();rd,wr=self.pipe()
        for fd,passed in [(wr,()),(rd,(rd,)),(2,()),(-1,()),(256,()),('bad',()),('3.0',())]:self.rejected(self.launch(self.command(fd),passed=passed))
        fd=os.open(self.key,os.O_RDONLY);self.fds.append(fd);self.rejected(self.launch(self.command(fd),passed=(fd,)))
    def test_blocking_pipe_rejected(self):
        self.ambient_pipe();rd,wr=self.pipe();os.set_blocking(wr,True);self.rejected(self.launch(self.command(wr),passed=(wr,)))
    def test_duplicate_mismatch_and_postsource_not_capability(self):
        self.ambient_pipe();rd,wr=self.pipe();a=self.command(wr);at=a.index('--');a[at:at]=['--native-metrics-fd',str(wr)];self.rejected(self.launch(a,passed=(wr,)))
        a=self.command(wr);a[a.index('--web-metrik-fd')+1]=str(wr+1);self.rejected(self.launch(a,passed=(wr,)))
        a=self.command(wr);at=a.index('--web-metrik-fd');pair=a[at:at+2];del a[at:at+2];a+=pair;self.rejected(self.launch(a,passed=(wr,)))
    def test_stdio_alias_rejected_before_private_source(self):
        self.ambient_pipe();rd,wr=self.pipe();r=self.launch(self.command(wr),passed=(wr,),stderr=wr);self.rejected(r);raw=b''
        try:raw=os.read(rd,512)
        except BlockingIOError:pass
        self.assertNotIn(b'CANARY',raw);self.assertFalse(self.marker.exists())
        r=self.launch(self.command(wr),passed=(wr,),stdout=wr);self.assertEqual(r.returncode,2);raw=os.read(rd,512);self.assertEqual(json.loads(raw)['reason'],'native_metrics_capability');self.assertNotIn(b'CANARY',raw);self.assertFalse(self.marker.exists())
    def test_sigpipe_disposition_rejected_without_restoration_change(self):
        self.ambient_pipe();rd,wr=self.pipe();old=signal.signal(signal.SIGPIPE,signal.SIG_DFL)
        try:
            with self.assertRaises(API['MetricsRejected']):API['metrics_capability'](self.command(wr)[4:])
            self.assertEqual(signal.getsignal(signal.SIGPIPE),signal.SIG_DFL)
        finally:signal.signal(signal.SIGPIPE,old)
if __name__=='__main__':unittest.main()
