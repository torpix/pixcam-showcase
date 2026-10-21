import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {createHash} from 'node:crypto';
const allowed=new Set([
 '.gitignore','Cargo.toml','Cargo.lock','LICENSE','README.md',
 'src/lib.rs','src/main.rs','src/core.rs','src/coordinates.rs','src/nc.rs','src/showcase.rs','src/solidcraft.rs','src/mcp.rs',
 'assets/index.html','assets/brand/concept-a-technical.svg','assets/brand/concept-b-optical.svg','assets/brand/concept-c-industrial.svg','assets/brand/pixcam-mark-dark.svg','assets/brand/pixcam-mark-light.svg','assets/brand/pixcam-horizontal.svg','assets/brand/favicon.svg','fixtures/aurora.nc',
 'tests/showcase.rs','tests/solidcraft.rs','tests/mcp.rs','scripts/browser-test.mjs','scripts/record.mjs','scripts/benchmark.mjs','scripts/privacy-check.mjs','scripts/schema-test.py',
 'docs/PRIVACY.md','docs/VERIFICATION.md','docs/SOLIDCRAFT.md','docs/MCP.md','docs/WALKTHROUGH.md','docs/BRAND.md','docs/BENCHMARK.md',
 'dist/index.html','dist/features.json','dist/features.csv','dist/feature-map.svg','dist/feature-map.dxf','dist/aurora.nc',
 'media/01-coordinate-inspector.png','media/02-feature-map.png','media/03-nc-inspector.png','media/pixcam-coordinate-inspector-72s.mp4',
 'src/bridge.rs','src/recognition.rs','src/local_server.rs','assets/bridge-panel.html','assets/bridge.css','assets/bridge.js',
 'fixtures/bridge-part.json','fixtures/bridge-part.solidcraft','fixtures/bridge-snapshot.json','fixtures/bridge-report.json',
 'schemas/solidcraft-snapshot.schema.json','schemas/feature-report.schema.json',
 'tests/bridge.rs','tests/recognition.rs','tests/local_server.rs','tests/mock_mcp.py',
 'scripts/capture.mjs','scripts/bridge-browser-test.mjs','scripts/live-gui-test.py','scripts/record-bridge.mjs','scripts/bridge-benchmark.py','scripts/geometry-schema-test.py',
 'docs/bridge-benchmark.json','media/04-solidcraft-bridge-demo.png','media/05-solidcraft-import-map.png','media/06-solidcraft-connected.png','media/pixcam-solidcraft-bridge-36s.mp4',
 'src/cycles.rs','src/registry.rs','tests/cycles.rs','assets/cycles-panel.html','assets/cycles.js',
 'fixtures/cycle-circular.json','fixtures/cycle-rectangular.json','scripts/cycles-browser-test.mjs','media/07-circular-cycle.png','media/08-rectangular-cycle-3plus2.png',
]);
const skip=new Set(['.git','target','test-results','node_modules']);
const banned=new RegExp(['\\b'+('88'+'5')+'\\b','h40'+'[17]','kost'+'ka','hell'+'er','R9010'+'15885','/'+'home/','/'+'mnt/','BACKUP_'+'H','-----BEGIN '+'(?:RSA |EC |OPENSSH )?PRIVATE KEY-----','gh[pousr]_'+'[A-Za-z0-9]{20,}','AKIA'+'[0-9A-Z]{16}'].join('|'),'i');
const fixtureHash=createHash('sha256').update(fs.readFileSync('fixtures/bridge-part.solidcraft')).digest('hex');
function scan(name,bytes){assert(!banned.test(name),`Blocked filename ${name}`);if(name.endsWith('.png')){assert.equal(bytes.subarray(0,8).toString('hex'),'89504e470d0a1a0a');return;}if(name.endsWith('.mp4')){assert.equal(bytes.subarray(4,8).toString(),'ftyp');return;}if(name.endsWith('.solidcraft')){assert.equal(createHash('sha256').update(bytes).digest('hex'),fixtureHash);return;}assert(!bytes.includes(0),`Unexpected binary ${name}`);const text=bytes.toString().replaceAll('/home/torpix/Aplikacje/solidcraft-cli-0.17.0','<PINNED_CLI>').replaceAll('/home/torpix/Aplikacje/solidcraft-0.17.0.AppImage','<PINNED_GUI>');assert(!banned.test(text),`Privacy marker in ${name}`);}
function walk(dir=''){return fs.readdirSync(dir||'.',{withFileTypes:true}).flatMap(e=>{if(!dir&&skip.has(e.name))return [];const p=path.posix.join(dir,e.name);assert(!e.isSymbolicLink(),`Symlink: ${p}`);return e.isDirectory()?walk(p):[p];});}
const files=walk();for(const file of files){assert(allowed.has(file),`Not in public allowlist: ${file}`);scan(file,fs.readFileSync(file));}for(const file of allowed)assert(files.includes(file),`Missing release file: ${file}`);
const d=JSON.parse(fs.readFileSync('dist/features.json'));assert.equal(d.part,'Aurora / synthetic inspection plate');assert.deepEqual(d.stock,{min_mm:[0,0,0],max_mm:[120,76,20]});assert.equal(d.features.length,9);assert.equal(d.program.motions.length,27);for(const f of d.features)assert.equal(f.source,'Original synthetic definition · src/showcase.rs · millimetres');assert.deepEqual(fs.readFileSync('fixtures/aurora.nc'),fs.readFileSync('dist/aurora.nc'));
const html=fs.readFileSync('dist/index.html','utf8');assert(!/https?:\/\/[^"']+\.(?:js|css|woff)/i.test(html));assert(!/__(?:DATA|EXPORTS)__/.test(html));assert(!/<script[^>]+src=/i.test(html));
const snapshot=JSON.parse(fs.readFileSync('fixtures/bridge-snapshot.json')),report=JSON.parse(fs.readFileSync('fixtures/bridge-report.json'));
assert.equal(snapshot.inspect_design.name,'bridge-part');assert.equal(snapshot.inspect_design.bodies.length,1);assert.equal(report.features.length,5);assert.equal(report.surfaces.length,24);
assert.equal(snapshot.inspect_design.path,undefined);assert.equal(snapshot.inspect_design.metadata,undefined);
assert(!/"(?:control_token|access_token|api_key|password)"\s*:/i.test(JSON.stringify({snapshot,report})));
// Inspect every JSON member of the one synthetic native CAD container. Only its known binary cache is allowed.
const archive=await new Promise((resolve,reject)=>{const p=spawn('python3',['-c',"import zipfile,json; z=zipfile.ZipFile('fixtures/bridge-part.solidcraft'); assert set(z.namelist())=={'document.json','container.json','cache/model.json','cache/bodies.bin'}; print(json.dumps({n:json.loads(z.read(n)) for n in z.namelist() if n.endswith('.json')}))"]);const chunks=[];p.stdout.on('data',b=>chunks.push(b));p.on('error',reject);p.on('close',code=>code===0?resolve(Buffer.concat(chunks)):reject(Error('Invalid synthetic CAD archive')));});scan('synthetic-native-json',archive);
async function git(...args){return new Promise((resolve,reject)=>{const p=spawn('git',args);const out=[];p.stdout.on('data',b=>out.push(b));p.on('error',reject);p.on('close',code=>code===0?resolve(Buffer.concat(out)):reject(new Error(`git ${args[0]} exit ${code}`)));});}
let objects=0;
if(process.argv.includes('--history')){
 assert.equal(fs.existsSync('.git/objects/info/alternates'),false);
 assert.equal((await git('remote')).toString().trim(),'');
 assert.equal((await git('rev-list','--all','--count')).toString().trim(),'5');
 assert.equal((await git('rev-list','--max-parents=0','--all','--count')).toString().trim(),'1','Public history must have one root');
 const head=(await git('rev-list','--parents','-n','1','HEAD')).toString().trim().split(' ');assert.equal(head.length,2,'Release must have exactly one public parent');
 assert.equal((await git('rev-parse','7ed8231^')).toString().trim().slice(0,7),'1cf208c','Reviewed v0.3 parent must remain v0.2');
 await git('merge-base','--is-ancestor','7ed8231','HEAD');
 assert.equal((await git('rev-list','--all','--merges')).toString().trim(),'','No foreign merge histories allowed');
 const tracked=(await git('ls-files','-z')).toString().split('\0').filter(Boolean);assert.deepEqual(new Set(tracked),allowed);
 const list=(await git('cat-file','--batch-all-objects','--batch-check=%(objectname) %(objecttype)')).toString().trim().split('\n');
 // Match allowed media blobs by exact content; every other object must pass the text scan.
 for(const row of list){const [oid,type]=row.split(' ');const bytes=await git('cat-file',type,oid);if(type==='blob'&&(bytes.subarray(0,8).toString('hex')==='89504e470d0a1a0a'||bytes.subarray(4,8).toString()==='ftyp')){objects++;continue;}if(type==='blob'&&bytes.subarray(0,4).toString('hex')==='504b0304'){assert.equal(createHash('sha256').update(bytes).digest('hex'),fixtureHash,'Only the reviewed synthetic native CAD ZIP is allowed');objects++;continue;}if(type==='tree'){assert(!banned.test(bytes.toString()),'Private tree path');}else scan(`git-${type}`,bytes);objects++;}
 assert.equal((await git('status','--porcelain')).toString().trim(),'','Public working tree must be clean');
}
console.log(`PASS privacy: ${files.length} allowlisted release files; synthetic contract; ${objects} Git objects checked. Media provenance requires visual review (see docs/PRIVACY.md).`);
