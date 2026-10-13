import concurrent.futures as cf, http.client, time, sys, collections
N=int(sys.argv[1]); C=int(sys.argv[2]); PORT=int(sys.argv[3])
def one(i):
    c=http.client.HTTPConnection("127.0.0.1",PORT,timeout=30)
    t=time.perf_counter()
    c.request("GET",f"/wiki/makale-{i}",headers={"Host":"itwise.test","Forwarded":f"for=203.0.113.{i%200+1};proto=https;host=itwise.test"})
    r=c.getresponse(); r.read(); c.close()
    return r.status, time.perf_counter()-t
t0=time.perf_counter()
with cf.ThreadPoolExecutor(C) as ex: res=list(ex.map(one,range(N)))
dt=time.perf_counter()-t0
lat=sorted(x[1] for x in res); st=collections.Counter(x[0] for x in res)
print(f"N={N} C={C} süre={dt:.2f}s  istek/sn={N/dt:.0f}  p50={lat[len(lat)//2]*1000:.1f}ms p95={lat[int(len(lat)*.95)]*1000:.1f}ms p99={lat[int(len(lat)*.99)]*1000:.1f}ms  durumlar={dict(st)}")
