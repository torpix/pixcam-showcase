"""Independent request schema + numerical engine validation and real CLI exports."""
import csv,io,json,subprocess,tempfile,xml.etree.ElementTree as ET
from pathlib import Path
from jsonschema import Draft202012Validator,ValidationError
schema=json.loads(Path('schemas/cycle-request.schema.json').read_text());Draft202012Validator.check_schema(schema)
validator=Draft202012Validator(schema)
for name,count in [('circular',8),('rectangular',12)]:
    request=json.loads(Path('fixtures/cycle-'+name+'.json').read_text());validator.validate(request)
    with tempfile.TemporaryDirectory(prefix='pixcam-cycle-test-') as tmp:
        r=json.loads(subprocess.check_output(['target/debug/pixcam-showcase','cycle','fixtures/cycle-'+name+'.json',tmp]))
        assert len(r['features'])==count and r['production_nc_generated'] is False
        assert json.loads(Path(tmp,'cycle.json').read_text())==r
        rows=list(csv.DictReader(io.StringIO(Path(tmp,'cycle.csv').read_text())))
        for f,row in zip(r['features'],rows):
            assert [float(row['global_'+axis+'_mm']) for axis in 'xyz']==f['global_xyz_mm']
            assert [float(row['local_'+axis+'_mm']) for axis in 'xyz']==f['local_xyz_mm']
            source=json.loads(row['source_parameters'])
            assert source==r['request']['pattern']
            assert all(source[key]==value for key,value in request['pattern'].items())
        root=ET.parse(Path(tmp,'cycle.svg')).getroot();assert len(root)==count+1
bad=json.loads(Path('fixtures/cycle-circular.json').read_text());bad['pattern']['pitch_diameter_mm']=50
try:validator.validate(bad)
except ValidationError:pass
else:raise AssertionError('Ambiguous pitch size accepted')
print('PASS: cycle request schemas, two actual CLI exports, exact JSON/CSV registry coordinates, source parameters and SVG structure.')
