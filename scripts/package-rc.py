"""Build a reviewed, commit-only public RC archive; never copies enclosing history/caches."""
import hashlib,io,json,re,subprocess,sys,zipfile
from pathlib import Path,PurePosixPath
subprocess.run(['node','scripts/privacy-check.mjs','--history'],check=True)
assert subprocess.check_output(['git','status','--porcelain']).strip()==b''
commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
prefix='pixcam-public-v0.4-rc1/'
data=subprocess.check_output(['git','archive','--format=zip','--prefix='+prefix,'HEAD'])
tracked=subprocess.check_output(['git','ls-files','-z']).decode().strip('\0').split('\0')
blocked=re.compile('|'.join([r'\b'+('88'+'5')+r'\b','h40'+'[17]','kost'+'ka','TC_'+'CARR','/'+'home/','/'+'mnt/','-----BEGIN '+'(?:RSA |EC |OPENSSH )?PRIVATE KEY-----',r'gh[pousr]_[A-Za-z0-9]{20,}',r'AKIA[0-9A-Z]{16}']),re.I)
with zipfile.ZipFile(io.BytesIO(data)) as archive:
    assert archive.testzip() is None
    files=[i for i in archive.infolist() if not i.is_dir()]
    assert len({i.filename for i in files})==len(files)==len(tracked)
    assert {i.filename.removeprefix(prefix) for i in files}==set(tracked)
    for item in files:
        p=PurePosixPath(item.filename);assert item.filename.startswith(prefix) and not p.is_absolute() and '..' not in p.parts
        name=item.filename.removeprefix(prefix)
        assert not any(x in p.parts for x in ['.git','target','node_modules','test-results','agent_docs'])
        body=archive.read(item);assert body==Path(name).read_bytes(),name
        if name.endswith(('.png','.mp4','.solidcraft')):continue # same bytes as reviewed release gate
        text=body.decode('utf-8')
        assert not blocked.search(name+'\n'+text),name
    html=archive.read(prefix+'dist/index.html').decode()
    assert '__CYCLES_' not in html and 'Parametric Cycles' in html and 'CYCLE_RECTANGULAR_PATTERN' in html
output=Path.cwd().parent/'pixcam-public-v0.4-rc1.zip'
output.write_bytes(data)
checksum=hashlib.sha256(data).hexdigest();output.with_suffix('.zip.sha256').write_text(checksum+'  '+output.name+'\n')
print(json.dumps({'archive':output.name,'source_commit':commit,'files':len(tracked),'bytes':len(data),'sha256':checksum,'privacy':'PASS; every ZIP member matched reviewed tracked bytes; no history or caches included'},indent=2))
