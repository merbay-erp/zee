#!/usr/bin/env python3
import json,os,runpy,tempfile,time,unittest
from pathlib import Path
API=runpy.run_path(str(Path(__file__).resolve().parents[1]/'native-metrics.py'))
MAX,decode,Collector,Channel=API['MAX'],API['decode'],API['Collector'],API['Channel']


def record(seq=1,kind=0,row=None,dropped=0):
    return (json.dumps({'schema':'zee-native-metrics-1','kind':kind,'seq':seq,'dropped':dropped,'v':([0]*(22 if kind==0 else 8) if row is None else row)},separators=(',',':'))+'\n').encode()


class Model(unittest.TestCase):
    def test_maximum_host_groups_fit_atomic_posix_log_record(self):
        for kind,row in [(0,[MAX]*11),(1,[MAX]*11),(2,[MAX]*8)]:
            value={'schema':'zee-worker-supervisor-1','event':'native_metrics','elapsed_ms':2147483647,'kind':kind,'v':row,'dropped':MAX,'unavailable':2,'rejected':MAX,'channels':2,'age_ms':2147483647}
            self.assertLessEqual(len(json.dumps(value,separators=(',',':')).encode())+1,512)
    def test_finite_bound(self):
        row=[MAX]*22
        self.assertLessEqual(len(record(MAX,row=row,dropped=MAX)),512)
        self.assertEqual(decode(record(MAX,row=row,dropped=MAX).strip())['v'],row)
    def test_no_extra_content(self):
        value=json.loads(record());value['path']='CANARY_PRIVATE';
        with self.assertRaises(ValueError):decode(json.dumps(value).encode())
    def test_counter_types_and_limits(self):
        for value in [True,-1,MAX+1,'CANARY_PRIVATE',1.2]:
            row=[0]*22;row[0]=value
            with self.assertRaises(ValueError):decode(record(row=row).strip())
    def test_duplicate_key(self):
        with self.assertRaises(ValueError):decode(b'{"schema":"a","schema":"b"}')
    def test_depth_and_size_bound(self):
        for value in [b'['*513,b'x'*513,b'[]']:
            with self.assertRaises((ValueError,RecursionError)):decode(value)
    def test_invalid_pool_state(self):
        for row in [[5,5,0,0,0,0,0,0],[4,5,0,0,0,0,0,0],[4,2,3,0,0,0,0,0]]:
            with self.assertRaises(ValueError):decode(record(kind=1,row=row).strip())
    def test_request_coherence(self):
        row=[0]*22;row[1]=1
        with self.assertRaises(ValueError):decode(record(row=row).strip())
    def test_split_frame_and_eof_partial(self):
        r,w=os.pipe();os.set_blocking(r,False);c=Channel(r)
        try:
            data=record();os.write(w,data[:15]);c.receive();self.assertEqual(c.seq[0],0)
            os.write(w,data[15:]);c.receive();self.assertEqual(c.seq[0],1)
            os.write(w,b'partial');os.close(w);w=None;c.receive();self.assertFalse(c.valid)
        finally:
            c.close()
            if w is not None:os.close(w)
    def test_out_of_order_is_per_kind(self):
        r,w=os.pipe();os.set_blocking(r,False);c=Channel(r)
        try:
            os.write(w,record(seq=3)+record(seq=2,kind=1,row=[1,1,0,0,1,0,5,6]));c.receive()
            self.assertEqual(c.seq,{0:3,1:2});self.assertEqual(c.rows[1][0],1)
        finally:c.close();os.close(w)
    def test_rollups_and_generation_reset(self):
        collector=Collector()
        for key,n in [(1,4),(2,7),(3,2)]:
            r,w=os.pipe();collector.attach(key,r)
            row=[0]*22;row[0]=row[1]=row[6]=n
            os.write(w,record(row=row));collector.collect();collector.finish(key);os.close(w)
        self.assertEqual(collector.snapshot()['native_request'][0],13)
        self.assertEqual(collector.snapshot()['native_pool'][:4],[0]*4)
    def test_only_two_owned_channels(self):
        c=Collector();fds=[]
        try:
            for key in (1,2):r,w=os.pipe();fds.append(w);c.attach(key,r)
            r,w=os.pipe();fds+=[r,w]
            with self.assertRaises(ValueError):c.attach(3,r)
        finally:c.close();[os.close(fd) for fd in fds]
    def test_decreasing_counter_rejects_channel(self):
        r,w=os.pipe();os.set_blocking(r,False);c=Channel(r)
        try:
            row=[0]*22;row[0]=2
            os.write(w,record(row=row)+record(seq=2));c.receive();self.assertFalse(c.valid)
        finally:c.close();os.close(w)
    def test_bounded_batch_and_buffer(self):
        r,w=os.pipe();os.set_blocking(r,False);os.set_blocking(w,False);c=Channel(r)
        try:
            sent=0
            while True:
                try:os.write(w,record(seq=sent+1));sent+=1
                except BlockingIOError:break
            c.receive();self.assertLessEqual(c.seq[0],64);self.assertLessEqual(len(c.buffer),4608)
        finally:c.close();os.close(w)
    def test_saturated_rollups_remain_finite(self):
        c=Collector();c.request=[MAX]*22
        for key in (1,2):
            r,w=os.pipe();c.attach(key,r);os.write(w,record(row=[MAX]*22));c.collect();c.finish(key);os.close(w)
        self.assertEqual(c.snapshot()['native_request'],[MAX]*22)


if __name__=='__main__':unittest.main()
