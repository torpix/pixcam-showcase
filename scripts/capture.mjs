// Captures only the isolated synthetic specimen. Generated artifacts are CLI outputs.
import fs from 'node:fs';
import {execFileSync} from 'node:child_process';
const binary='target/debug/pixcam-showcase';
const snapshot=execFileSync(binary,['capture-solidcraft','--in','fixtures/bridge-part.solidcraft'],{maxBuffer:16*1024*1024,timeout:60000});
const parsed=JSON.parse(snapshot);
if(parsed.inspect_design.name!=='bridge-part')throw Error('Unexpected specimen');
fs.writeFileSync('fixtures/bridge-snapshot.json',snapshot);
const report=execFileSync(binary,['analyze-solidcraft','fixtures/bridge-snapshot.json'],{maxBuffer:16*1024*1024});
fs.writeFileSync('fixtures/bridge-report.json',report);
console.log('Captured actual SolidCraft 0.17.0 geometry through read-only MCP.');
