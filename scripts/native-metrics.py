#!/usr/bin/env python3
"""K-257 bounded consumer for owned native counter pipes; no payload passthrough."""
import json,os,time
MAX=9007199254740991
SCHEMA='zee-native-metrics-1'


def add(a,b):return min(MAX,a+b)


def unique(rows):
    result={}
    for key,value in rows:
        if key in result:raise ValueError('duplicate')
        result[key]=value
    return result


def decode(data):
    if not 0<len(data)<=512:raise ValueError('size')
    v=json.loads(data,object_pairs_hook=unique,parse_constant=lambda _:(_ for _ in ()).throw(ValueError('constant')))
    if not isinstance(v,dict) or set(v)!={'schema','kind','seq','dropped','v'} or v['schema']!=SCHEMA:raise ValueError('schema')
    if type(v['kind']) is not int or v['kind'] not in (0,1):raise ValueError('kind')
    for key in ('seq','dropped'):
        if type(v[key]) is not int or not 0<=v[key]<=MAX:raise ValueError('counter')
    if v['seq']==0:raise ValueError('seq')
    row=v['v']
    if not isinstance(row,list) or len(row)!=(22 if v['kind']==0 else 8) or any(type(n) is not int or not 0<=n<=MAX for n in row):raise ValueError('row')
    if v['kind']==0:
        if row[1]>row[0] or min(MAX,sum(row[6:]))>row[1] or min(MAX,row[3]+row[4])>row[1]:raise ValueError('request')
    elif row[0]>4 or row[1]>row[0] or row[2]!=row[0]-row[1]:raise ValueError('pool')
    return v


class Channel:
    def __init__(self,fd):
        self.fd=fd;self.buffer=bytearray();self.rows={0:[0]*22,1:[0]*8};self.seq={0:0,1:0};self.dropped=0;self.last={0:None,1:None};self.valid=True
    def close(self):
        if self.fd is not None:os.close(self.fd);self.fd=None
    def receive(self):
        # At most64 parsed frames and64KiB bytes per call; no blocking read.
        frames=0;read_bytes=0
        try:
            while frames<64 and read_bytes<65536:
                if b'\n' not in self.buffer:
                    if len(self.buffer)>512:raise ValueError('size')
                    try:part=os.read(self.fd, min(4096,65536-read_bytes))
                    except BlockingIOError:return
                    if not part:
                        if self.buffer:raise ValueError('partial')
                        self.close();return
                    read_bytes+=len(part);self.buffer.extend(part)
                while b'\n' in self.buffer and frames<64:
                    raw,_,rest=self.buffer.partition(b'\n');self.buffer=bytearray(rest);v=decode(raw);frames+=1
                    kind=v['kind']
                    self.dropped=max(self.dropped,v['dropped'])
                    if v['seq']<=self.seq[kind]:continue
                    # Cumulative counters cannot decrease within this owned generation.
                    counter_start=0 if kind==0 else 4
                    if any(new<old for new,old in zip(v['v'][counter_start:],self.rows[kind][counter_start:])):raise ValueError('counter-decrease')
                    self.rows[kind]=v['v'];self.seq[kind]=v['seq'];self.last[kind]=time.monotonic()
        except (ValueError,TypeError,RecursionError,OSError):self.valid=False;self.close()


class Collector:
    def __init__(self):
        self.channels={};self.request=[0]*22;self.pool=[0]*8;self.dropped=0;self.rejected=0
    def attach(self,key,fd):
        if key in self.channels or len(self.channels)>=2:raise ValueError('ownership')
        os.set_blocking(fd,False);self.channels[key]=Channel(fd)
    def collect(self):
        for channel in self.channels.values():
            if channel.fd is not None:channel.receive()
    def finish(self,key):
        channel=self.channels.pop(key,None)
        if channel is None:return
        if channel.fd is not None:channel.receive()
        channel.close()
        self.request=[add(a,b) for a,b in zip(self.request,channel.rows[0])]
        for index in range(4,8):self.pool[index]=add(self.pool[index],channel.rows[1][index])
        self.dropped=add(self.dropped,channel.dropped)
        if not channel.valid:self.rejected=add(self.rejected,1)
    def snapshot(self):
        self.collect();request=self.request.copy();pool=self.pool.copy();dropped=self.dropped;unavailable=0;ages={0:[],1:[]}
        for channel in self.channels.values():
            request=[add(a,b) for a,b in zip(request,channel.rows[0])]
            pool=[add(a,b) for a,b in zip(pool,channel.rows[1])]
            dropped=add(dropped,channel.dropped)
            if not channel.valid or channel.fd is None or channel.seq[0]==0:unavailable+=1
            for kind in (0,1):
                if channel.last[kind] is not None:ages[kind].append(min(2147483647,int((time.monotonic()-channel.last[kind])*1000)))
        return {'native_request':request,'native_pool':pool,'native_snapshot_drops':dropped,'native_metric_channels':len(self.channels),'native_metric_unavailable':unavailable,'native_metric_rejected_total':self.rejected,'native_request_age_ms':max(ages[0],default=0),'native_pool_age_ms':max(ages[1],default=0)}
    def close(self):
        for key in list(self.channels):self.finish(key)
