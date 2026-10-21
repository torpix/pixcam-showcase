"""Benchmark only the public synthetic specimen; generated JSON is public safe."""
import json,platform,statistics,subprocess,time
from pathlib import Path
binary='target/release/pixcam-showcase'
result=json.loads(subprocess.check_output([binary,'benchmark-solidcraft','fixtures/bridge-snapshot.json']))
samples=[]
for _ in range(10):
    start=time.perf_counter()
    data=json.loads(subprocess.check_output([binary,'capture-solidcraft','--in','fixtures/bridge-part.solidcraft'],timeout=60))
    assert data['inspect_design']['name']=='bridge-part'
    samples.append((time.perf_counter()-start)*1000)
samples.sort()
result['headless_capture']={'runs':10,'median_ms':statistics.median(samples),'p95_ms':samples[-1],
    'scope':'fresh CLI launch, native synthetic document loading, initialize, four read-only tool types, consistency check and recognition'}
result['platform']=platform.system()+' '+platform.machine()
Path('docs/bridge-benchmark.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
