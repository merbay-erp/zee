import concurrent.futures as cf, http.client, time, sys
N=int(sys.argv[1]); C=int(sys.argv[2]); ports=[int(p) for p in sys.argv[3].split(",")]; off=int(sys.argv[4])
def one(i):
    c=http.client.HTTPConnection("127.0.0.1",ports[i%len(ports)],timeout=60)
    c.request("GET",f"/wiki/m-{off+i}",headers={"Host":"itwise.test","Forwarded":f"for=198.51.100.{i%200+1};proto=https;host=itwise.test"})
    r=c.getresponse(); r.read(); c.close(); return r.status
t0=time.perf_counter()
with cf.ThreadPoolExecutor(C) as ex: st=list(ex.map(one,range(N)))
dt=time.perf_counter()-t0
print(f"workers={len(ports)} N={N} istek/sn={N/dt:.0f} 200={st.count(200)} diğer={len(st)-st.count(200)}")
