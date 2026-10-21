import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
const module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
fs.mkdirSync('test-results',{recursive:true});fs.mkdirSync('media',{recursive:true});
const page=await browser.newPage({viewport:{width:1600,height:1200}}),errors=[],requests=[];
page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>requests.push(r.url()));
let adapter;
const snapshot=JSON.parse(fs.readFileSync('fixtures/bridge-snapshot.json'));
const expected=JSON.parse(fs.readFileSync('fixtures/bridge-report.json'));
try {
  await page.goto(pathToFileURL(path.resolve('dist/index.html')).href);
  assert.equal(await page.locator('.bridge-feature').count(),5);assert.match(await page.locator('#bridge-status').textContent(),/DEMO.*offline/);
  const parity=await page.evaluate(s=>window.pixcamBridge.analyze(s),snapshot);
  assert.deepEqual(parity.features,expected.features);assert.deepEqual(parity.groups,expected.groups);assert.deepEqual(parity.coaxial_groups,expected.coaxial_groups);assert.deepEqual(parity.potential_operations,expected.potential_operations);
  await page.screenshot({path:'media/04-solidcraft-bridge-demo.png',fullPage:true});
  await page.click('[data-feature="CY05"]');await page.selectOption('#bridge-setup','1');
  await page.waitForTimeout(700);assert.match(await page.locator('#bridge-details').textContent(),/Parallel \/ access requires review/);
  await page.selectOption('#bridge-filter','diameter');assert.match(await page.locator('#bridge-groups').textContent(),/6.00000.*CY01, CY04/);
  await page.selectOption('#bridge-filter','coaxial');assert.match(await page.locator('#bridge-groups').textContent(),/CY03, CY04/);
  await page.click('#bridge-import-button');await page.locator('#bridge-file').setInputFiles('fixtures/bridge-snapshot.json');
  await page.waitForFunction(()=>document.querySelector('#bridge-status').textContent.includes('analyzed locally'));
  assert.equal(await page.locator('.bridge-feature').count(),5);
  await page.click('#bridge-map');await page.screenshot({path:'media/05-solidcraft-import-map.png',fullPage:true});
  const invalid=structuredClone(snapshot);invalid.body_topology[0].faces[4].cylinder.radius=-1;
  await page.locator('#bridge-file').setInputFiles({name:'invalid.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify(invalid))});
  await page.waitForFunction(()=>document.querySelector('#bridge-status').textContent.startsWith('IMPORT failed'));
  assert.equal(await page.locator('.bridge-feature').count(),5);assert.equal(await page.evaluate(()=>window.pixcamBridge.stale),true);
  await page.locator('#bridge-file').setInputFiles('fixtures/bridge-snapshot.json');await page.waitForFunction(()=>!window.pixcamBridge.stale);
  const partial=structuredClone(snapshot);partial.body_topology[0].edges=[];
  assert.equal(await page.evaluate(s=>window.pixcamBridge.analyze(s).features.every(f=>f.boundary_span_mm===null),partial),true);
  for(const [id,name]of [['bridge-json','pixcam-report.json'],['bridge-csv','pixcam-registry.csv'],['bridge-snapshot','solidcraft-snapshot.json'],['bridge-svg','pixcam-surface-map.svg'],['bridge-png','pixcam-cad-view.png']]){
    const promise=page.waitForEvent('download');await page.click('#'+id);const d=await promise;assert.equal(d.suggestedFilename(),name);await d.saveAs('test-results/'+name);
  }
  assert.deepEqual(JSON.parse(fs.readFileSync('test-results/pixcam-report.json')).features,expected.features);
  assert.equal(fs.readFileSync('test-results/pixcam-registry.csv','utf8').trim().split('\n').length,6);
  await page.click('#bridge-connected-button');await page.click('#bridge-refresh');assert.match(await page.locator('#bridge-status').textContent(),/unavailable/);
  assert.equal(requests.filter(x=>/^https?:/.test(x)).length,0,'offline modes must not use network');
  await page.setViewportSize({width:390,height:844});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  await page.setViewportSize({width:1600,height:1200});
  adapter=spawn('target/debug/pixcam-showcase',['bridge',...(process.env.SOLIDCRAFT_TEST_CONNECT?['--connect',process.env.SOLIDCRAFT_TEST_CONNECT]:['--in','fixtures/bridge-part.solidcraft'])],{stdio:['ignore','pipe','pipe']});
  const url=await new Promise((resolve,reject)=>{adapter.stdout.once('data',b=>resolve(b.toString().match(/http:\/\/127\.0\.0\.1:\d+\//)[0]));adapter.once('error',reject);adapter.once('exit',code=>reject(Error('Adapter exited '+code)));});
  await page.goto(url);await page.click('#bridge-connected-button');await page.click('#bridge-refresh');
  await page.waitForFunction(()=>document.querySelector('#bridge-status').textContent.includes('/ refreshed'),{},{timeout:65000});
  assert.equal(await page.locator('.bridge-feature').count(),5);
  const active=await page.evaluate(()=>window.pixcamBridge.report);
  assert.equal(active.document,'bridge-part');for(let i=0;i<5;i++)assert(Math.abs(active.features[i].diameter_mm-expected.features[i].diameter_mm)<1e-6);
  await page.screenshot({path:'media/06-solidcraft-connected.png',fullPage:true});
  // Real HTTP rejection tests: no token, hostile origin, Host DNS rebinding, raw tool routes.
  for(const headers of [{},{Origin:'https://evil.example','Content-Type':'application/json'},{Host:'evil.example'}]){
    // Keep the unauthorized body empty: closing a TCP socket with an unread body can surface as
    // ECONNRESET in newer Node/undici releases even though the adapter correctly denied it.
    const r=await fetch(url+'api/refresh',{method:'POST',headers,body:''});assert.equal(r.status,403);assert.equal(r.headers.get('access-control-allow-origin'),null);
  }
  assert.equal((await fetch(url+'api/execute',{method:'POST',body:''})).status,403);
  await page.click('#bridge-refresh');await page.waitForFunction(()=>!document.querySelector('#bridge-refresh').disabled,{},{timeout:65000});
  assert.match(await page.locator('#bridge-status').textContent(),/refreshed/);
  fs.writeFileSync('test-results/browser-connected.json',JSON.stringify({mode:process.env.SOLIDCRAFT_TEST_CONNECT?'GUI':'HEADLESS',document:active.document,feature_count:active.features.length,body_count:active.bodies.length,passed:true},null,2));
  adapter.kill();await once(adapter,'exit');adapter=null;
  await page.click('#bridge-refresh');await page.waitForFunction(()=>document.querySelector('#bridge-status').textContent.startsWith('DISCONNECTED'));
  assert.equal(await page.evaluate(()=>window.pixcamBridge.stale),true);assert.equal(await page.locator('.bridge-feature').count(),5);
  assert.deepEqual(errors,[]);
  console.log('PASS: offline DEMO/IMPORT, Rust/JS recognition parity, all exports, partial/invalid data, axes/3+2, responsive UI, real MCP CONNECTED refresh, HTTP origin/Host/token rejection, disconnect/stale handling.');
} finally {if(adapter)adapter.kill();await browser.close();}
