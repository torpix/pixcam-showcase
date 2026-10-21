"""Validate the neutral manufacture-adapter v1 example and fail-closed cases."""
import copy
import json
from pathlib import Path
from jsonschema import Draft202012Validator, ValidationError

schema = json.loads(Path("schemas/manufacture-adapter.schema.json").read_text())
example = json.loads(Path("fixtures/manufacture-adapter-v1.json").read_text())
Draft202012Validator.check_schema(schema)
validator = Draft202012Validator(schema)
validator.validate(example)

for mutate in (
    lambda value: value.update(units="inch"),
    lambda value: value["coordinate_frames"][0].update(handedness="left"),
    lambda value: value["manufacturing_features"][0]["status"].update(evidence="CERTIFIED_TOOLPATH"),
    lambda value: value["position_patterns"][0].update(positions=[]),
):
    invalid = copy.deepcopy(example)
    mutate(invalid)
    try:
        validator.validate(invalid)
    except ValidationError:
        pass
    else:
        raise AssertionError("Invalid manufacture adapter payload accepted")

print("PASS: manufacture-adapter/1 schema, synthetic example, units/handedness/evidence/empty-pattern rejection.")
