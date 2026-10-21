import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawn} from 'node:child_process';
const allowed=new Set([
 '.gitignore','Cargo.toml','Cargo.lock','LICENSE','README.md',
 'src/lib.rs','src/main.rs','src/core.rs','src/nc.rs','src/showcase.rs','src/mcp.rs',
 'assets/index.html','fixtures/aurora.nc',
 'tests/showcase.rs','tests/mcp.rs','scripts/browser-test.mjs','scripts/record.mjs','scripts/privacy-check.mjs','scripts/schema-test.py',
 'docs/PRIVACY.md','docs/VERIFICATION.md','docs/SOLIDCRAFT.md','docs/MCP.md','docs/WALKTHROUGH.md',
 'dist/index.html','dist/features.json','dist/features.csv','dist/feature-map.svg','dist/feature-map.dxf','dist/aurora.nc',
 'media/01-feature-map.png','media/02-orientation-groups.png','media/03-nc-preview.png','media/pixcam-aurora-72s.mp4',
]);
const skip=new Set(['.git','target','test-results','node_modules']);
const banned=new RegExp(['\\b'+('88'+'5')+'\\b','h40'+'[17]','kost'+'ka','hell'+'er','R9010'+'15885','/'+'home/','/'+'mnt/','BACKUP_'+'H','-----BEGIN '+'(?:RSA |EC |OPENSSH )?PRIVATE KEY-----','gh[pousr]_'+'[A-Za-z0-9]{20,}','AKIA'+'[0-9A-Z]{16}'].join('|'),'i');
function scan(name,bytes){assert(!banned.test(name),`Blocked filename ${name}`);if(name.endsWith('.png')){assert.equal(bytes.subarray(0,8).toString('hex'),'89504e470d0a1a0a');return;}if(name.endsWith('.mp4')){assert.equal(bytes.subarray(4,8).toString(),'ftyp');return;}assert(!bytes.includes(0),`Unexpected binary ${name}`);assert(!banned.test(bytes.toString()),`Privacy marker in ${name}`);}
function walk(dir=''){return fs.readdirSync(dir||'.',{withFileTypes:true}).flatMap(e=>{if(!dir&&skip.has(e.name))return [];const p=path.posix.join(dir,e.name);assert(!e.isSymbolicLink(),`Symlink: ${p}`);return e.isDirectory()?walk(p):[p];});}
const files=walk();for(const file of files){assert(allowed.has(file),`Not in public allowlist: ${file}`);scan(file,fs.readFileSync(file));}for(const file of allowed)assert(files.includes(file),`Missing release file: ${file}`);
const d=JSON.parse(fs.readFileSync('dist/features.json'));assert.equal(d.part,'Aurora / synthetic inspection plate');assert.deepEqual(d.stock,{min_mm:[0,0,0],max_mm:[120,76,20]});assert.equal(d.features.length,9);assert.equal(d.program.motions.length,27);for(const f of d.features)assert.equal(f.source,'Original synthetic definition · src/showcase.rs · millimetres');assert.deepEqual(fs.readFileSync('fixtures/aurora.nc'),fs.readFileSync('dist/aurora.nc'));
const html=fs.readFileSync('dist/index.html','utf8');assert(!/https?:\/\/[^"']+\.(?:js|css|woff)/i.test(html));assert(!/__(?:DATA|EXPORTS)__/.test(html));assert(!/<script[^>]+src=/i.test(html));
async function git(...args){return new Promise((resolve,reject)=>{const p=spawn('git',args);const out=[];p.stdout.on('data',b=>out.push(b));p.on('error',reject);p.on('close',code=>code===0?resolve(Buffer.concat(out)):reject(new Error(`git ${args[0]} exit ${code}`)));});}
let objects=0;
if(process.argv.includes('--history')){
 assert.equal(fs.existsSync('.git/objects/info/alternates'),false);
 assert.equal((await git('remote')).toString().trim(),'');
 assert.equal((await git('rev-list','--all','--count')).toString().trim(),'1');
 const parents=(await git('rev-list','--parents','HEAD')).toString().trim().split(' ');assert.equal(parents.length,1,'Public commit must have no parent');
 const tracked=(await git('ls-files','-z')).toString().split('\0').filter(Boolean);assert.deepEqual(new Set(tracked),allowed);
 const list=(await git('cat-file','--batch-all-objects','--batch-check=%(objectname) %(objecttype)')).toString().trim().split('\n');
 // Match allowed media blobs by exact content; every other object must pass the text scan.
 const media=new Set();for(const f of files.filter(f=>/\.(png|mp4)$/.test(f)))media.add((await git('hash-object',f)).toString().trim());
 for(const row of list){const [oid,type]=row.split(' ');if(media.has(oid)){objects++;continue;}const bytes=await git('cat-file',type,oid);if(type==='tree'){assert(!banned.test(bytes.toString()),'Private tree path');}else scan(`git-${type}`,bytes);objects++;}
 assert.equal((await git('status','--porcelain')).toString().trim(),'','Public working tree must be clean');
}
console.log(`PASS privacy: ${files.length} allowlisted release files; synthetic contract; ${objects} Git objects checked. Media provenance requires visual review (see docs/PRIVACY.md).`);
