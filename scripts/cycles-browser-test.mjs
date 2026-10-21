import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
const run=promisify(execFile),module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
const page=await browser.newPage({viewport:{width:1600,height:1100}}),errors=[],network=[];
page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(/^https?:/.test(r.url()))network.push(r.url());});
fs.mkdirSync('test-results',{recursive:true});fs.mkdirSync('media',{recursive:true});
const close=(a,b)=>assert(Math.abs(a-b)<1e-8,`${a} != ${b}`);
try{
 await page.goto(pathToFileURL(path.resolve('dist/index.html')).href);
 assert.equal(await page.locator('#cycle-point option').count(),8);
 for(const name of ['circular','rectangular']){
   const q=JSON.parse(fs.readFileSync('fixtures/cycle-'+name+'.json'));
   const {stdout}=await run('target/debug/pixcam-showcase',['cycle','fixtures/cycle-'+name+'.json'],{maxBuffer:4*1024*1024});const rust=JSON.parse(stdout);
   const js=await page.evaluate(q=>window.pixcamCycles.generate(q),q);
   assert.equal(js.features.length,rust.features.length);
   for(let i=0;i<js.features.length;i++)for(const key of ['local_xyz_mm','global_xyz_mm','tool_axis_global'])js.features[i][key].forEach((v,j)=>close(v,rust.features[i][key][j]));
 }
 await page.screenshot({path:'media/07-circular-cycle.png',fullPage:false});
 await page.locator('#cycle-count').fill('4');await page.locator('#cycle-sweep').fill('90');await page.selectOption('#cycle-direction','clockwise');
 let r=await page.evaluate(()=>window.pixcamCycles.result);assert.equal(r.closed_pattern,false);close(r.features[3].local_xyz_mm[1],-25);
 await page.selectOption('#cycle-frame','1');r=await page.evaluate(()=>window.pixcamCycles.result);close(r.features[0].tool_axis_global[0],-1);
 await page.selectOption('#cycle-kind','rectangular');assert.equal(await page.locator('#cycle-point option').count(),12);
 await page.locator('#cycle-rotation').fill('90');r=await page.evaluate(()=>window.pixcamCycles.result);close(r.features[11].local_xyz_mm[0],-70);close(r.features[11].local_xyz_mm[1],40);
 await page.selectOption('#cycle-frame','3');await page.selectOption('#cycle-point','11');assert.match(await page.locator('#cycle-coordinates').textContent(),/GRID \/ 11/);
 await page.screenshot({path:'media/08-rectangular-cycle-3plus2.png',fullPage:false});
 await page.locator('#cycle-spacing-x').fill('0');assert.match(await page.locator('#cycle-status').textContent(),/INVALID/);assert.equal(await page.locator('#cycle-json').isDisabled(),true);
 await page.locator('#cycle-spacing-x').fill('20');assert.equal(await page.locator('#cycle-json').isDisabled(),false);
 await page.selectOption('#cycle-frame','custom');await page.locator('#cycle-frame-matrix').fill('[[2,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]');assert.equal(await page.evaluate(()=>window.pixcamCycles.valid),false);
 const matrix=[[0,-1,0,11],[1,0,0,23],[0,0,1,7],[0,0,0,1]];
 await page.locator('#cycle-frame-matrix').fill(JSON.stringify(matrix));assert.equal(await page.evaluate(()=>window.pixcamCycles.valid),true);
 r=await page.evaluate(()=>window.pixcamCycles.result);
 for(const type of ['json','csv','svg','dxf']){const promise=page.waitForEvent('download');await page.click('#cycle-'+type);const d=await promise;assert.equal(d.suggestedFilename(),'cycle.'+type);await d.saveAs('test-results/cycle.'+type);}
 const json=JSON.parse(fs.readFileSync('test-results/cycle.json'));assert.deepEqual(json.features,r.features);
 const csv=fs.readFileSync('test-results/cycle.csv','utf8');for(const [i,line]of csv.trim().split('\n').slice(1).entries()){const fields=line.split(',');for(let j=0;j<3;j++){close(Number(fields[3+j].replaceAll('"','')),r.features[i].local_xyz_mm[j]);close(Number(fields[6+j].replaceAll('"','')),r.features[i].global_xyz_mm[j]);}}
 assert.equal(fs.readFileSync('test-results/cycle.svg','utf8').match(/<circle /g).length,12);assert.equal(fs.readFileSync('test-results/cycle.dxf','utf8').match(/\nPOINT\n/g).length,12);
 const promise=page.waitForEvent('download');await page.click('#cycle-registry');await (await promise).saveAs('test-results/combined-registry.json');
 const registry=JSON.parse(fs.readFileSync('test-results/combined-registry.json'));assert.equal(registry.imported_cad.features.length,5);assert.equal(registry.feature_count,17);assert.deepEqual(registry.parametric_cycles[0].features,r.features);
 const last=await page.evaluate(()=>window.pixcamCycles.result);last.request.pattern.rows=1000;last.request.pattern.columns=1000;
 assert.equal(await page.evaluate(q=>{try{window.pixcamCycles.generate(q);return false;}catch{return true;}},last.request),true);
 await page.setViewportSize({width:390,height:844});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
 assert.deepEqual(network,[]);assert.deepEqual(errors,[]);
 console.log('PASS cycles: Rust/browser parity, circular/open/CW, 4×3 rotated grid, 3+2/custom frames, local/global inspection, invalid-input blocking, four exports and combined CAD registry, mobile and zero network/errors.');
}finally{await browser.close();}
