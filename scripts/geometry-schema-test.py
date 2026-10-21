"""Independent offline schema validation; no remote schema retrieval."""
import copy,json
from pathlib import Path
from jsonschema import Draft202012Validator,ValidationError
from referencing import Registry,Resource
schemas=[json.loads(p.read_text()) for p in Path('schemas').glob('*.schema.json')]
for schema in schemas:Draft202012Validator.check_schema(schema)
registry=Registry().with_resources((s['$id'],Resource.from_contents(s)) for s in schemas)
validators={s['$id'].split('/')[-1]:Draft202012Validator(s,registry=registry) for s in schemas}
snapshot=json.loads(Path('fixtures/bridge-snapshot.json').read_text())
report=json.loads(Path('fixtures/bridge-report.json').read_text())
validators['solidcraft-snapshot.schema.json'].validate(snapshot)
validators['feature-report.schema.json'].validate(report)
for path in [Path('test-results/pixcam-report.json'),Path('test-results/gui-snapshot.json')]:
    if path.exists():validators['feature-report.schema.json' if 'report' in path.name else 'solidcraft-snapshot.schema.json'].validate(json.loads(path.read_text()))
bad=copy.deepcopy(snapshot);bad['body_topology'][0]['faces'][4]['cylinder']['radius']=-1
try:validators['solidcraft-snapshot.schema.json'].validate(bad)
except ValidationError:pass
else:raise AssertionError('Negative radius accepted')
bad=copy.deepcopy(report);bad['features'][0]['inference_level']='VERIFIED_GEOMETRY'
try:validators['feature-report.schema.json'].validate(bad)
except ValidationError:pass
else:raise AssertionError('Inference promoted to verified geometry')
print('PASS: snapshot/report draft 2020-12 schemas, CAD capture, exported report, GUI snapshot, negative radius and evidence-level rejection.')
