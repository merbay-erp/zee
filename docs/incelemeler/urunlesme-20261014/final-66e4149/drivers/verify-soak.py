"""Actual final receipt/raw sample verification only, run after owned soak terminates."""
import hashlib,json,re,subprocess
from pathlib import Path
BASE=Path(__file__).resolve().parent;OUT=BASE.parent/'s3-66e4149-soak';S='66e4149e53451111d153f47283c219079a8d1cab';TREE='936eb74d6ce14b1bf9709f03756e0184d7428fa2'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
r=json.loads((OUT/'PROOF.json').read_text());assert r['status']=='passed' and r['acceptance'] is True and r['source_sha']==S and r['actual_elapsed_seconds']>=1800 and r['seconds']==1800 and r['window_seconds']==60
assert r['threshold_percent']==10 and r['threshold_kib']==32768 and r['threshold_operator']=='both-exceeded';assert r['binary_start_sha256']==r['binary_end_sha256'] and r['external_runner_start_sha256']==r['external_runner_end_sha256']
for name,h in r['binary_end_sha256'].items():assert sha(OUT/'target/release'/name)==h
assert sha(OUT/'soak-history.tsv')==r['history_sha256'] and sha(OUT/'soak-report.md')==r['report_sha256']
for cmd in r['commands']:assert sha(OUT/cmd['log'])==cmd['log_sha256']
assert r['source_end_clean'] is True and r['source_end_sha']==S
for repo in [OUT/'source',Path('/Users/mustafaerbay/IdeaProjects/zee-urunlesme-20261014')]:
 def git(*args):return subprocess.check_output(['git','-C',str(repo),*args],timeout=20).decode().strip()
 assert git('rev-parse','HEAD')==S and git('rev-parse','HEAD^{tree}')==TREE and git('status','--porcelain','--untracked-files=all')==''
report=(OUT/'soak-report.md').read_text();assert 'Süre 1800 sn, pencere 60 sn' in report and 'Sonuç: **GEÇTİ**' in report
rows=[tuple(map(int,m)) for m in re.findall(r'^\| ([0-9]+) \| ([0-9]+) \| ([0-9]+) \| ([0-9]+) \| ([0-9]+) \|$',report,re.M)];assert len(rows)>=3 and all(all(x>0 for x in row) for row in rows)
assert all(a[0]<=b[0] and a[3]<=b[3] and a[4]<=b[4] for a,b in zip(rows,rows[1:]));windows={}
for row in rows:windows.setdefault(row[0]//60,[]).append(row)
ordered=[windows[k] for k in sorted(windows)];assert len(ordered)>=3;first,last=ordered[1],ordered[-1]
median=lambda xs:sorted(xs)[len(xs)//2]
values=[]
for column in [1,2]:
 a=median([x[column] for x in first]);b=median([x[column] for x in last]);pct=0 if b<=a or a==0 else ((b-a)*100+a-1)//a;assert not(pct>10 and max(0,b-a)>32768);values.append({'first_kib':a,'last_kib':b,'growth_percent':pct,'growth_kib':max(0,b-a)})
row=[x.split('\t') for x in (OUT/'soak-history.tsv').read_text().splitlines() if x and not x.startswith('#')];assert len(row)==1 and len(row[0])==17;h=row[0];assert h[0]==S and h[4:6]==['1800','60'] and h[14:]==['10','32768','gecti'];assert list(map(int,h[8:12]))==[values[0]['first_kib'],values[0]['last_kib'],values[1]['first_kib'],values[1]['last_kib']] and list(map(int,h[12:14]))==[v['growth_percent'] for v in values];assert list(map(int,h[6:8]))==list(rows[-1][3:5])
summary={'status':'passed','source_sha':S,'tree':TREE,'proof_sha256':sha(OUT/'PROOF.json'),'actual_seconds':r['actual_elapsed_seconds'],'requested_seconds':1800,'window_seconds':60,'actual_raw_samples':len(rows),'compiler':values[0],'dillsp':values[1],'compiler_periods':rows[-1][3],'lsp_periods':rows[-1][4],'raw_sample_median_threshold_recomputed':True,'history_sha256':r['history_sha256'],'report_sha256':r['report_sha256'],'binary_pins_unchanged':True,'original_and_clone_end_clean':True,'threshold':'both >10percent AND >32768KiB rejects, unchanged','boundaries':'Actual exactS local1800 soak only, not RC/900/canonicaldelivery/install/Linux/P08/human/platform/release acceptance'}
(BASE/'S3-SOAK-RESULT.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({'summary_sha256':sha(BASE/'S3-SOAK-RESULT.json'),**summary},indent=2))
