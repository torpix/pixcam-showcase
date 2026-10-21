import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
const module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
const browser=await chromium.launch({executablePath:process.env.CHROMIUM||'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
fs.mkdirSync('media',{recursive:true});fs.mkdirSync('test-results',{recursive:true});
const page=await browser.newPage({viewport:{width:1440,height:1080},deviceScaleFactor:1});
const errors=[],network=[];page.on('pageerror',e=>errors.push(e.message));page.on('request',r=>{if(/^https?:/.test(r.url()))network.push(r.url());});
try{
 await page.goto(pathToFileURL(path.resolve('dist/index.html')).href);
 assert.equal(await page.locator('.feature').count(),9);
 await page.screenshot({path:'media/01-feature-map.png',fullPage:true});
 await page.selectOption('#face','Z_MAX');assert.equal(await page.locator('.feature').count(),6);
 await page.selectOption('#diameter','6');assert.equal(await page.locator('.feature').count(),3);
 await page.selectOption('#face','Y_MIN');assert.equal(await page.locator('.feature').count(),2);
 await page.selectOption('#face','X_MAX');assert.equal(await page.locator('.feature').count(),0);assert.equal(await page.locator('#feature-title').textContent(),'No features');
 await page.click('#reset');await page.click('[data-id="S01"]');assert.match(await page.locator('#details').textContent(),/Not circular/);
 await page.click('[data-id="P01"]');assert.match(await page.locator('#details').textContent(),/73, 46, 20/);
 await page.click('#groups-tab');assert.equal(await page.locator('.group').count(),5);
 await page.screenshot({path:'media/02-orientation-groups.png',fullPage:true});
 await page.click('#nc-tab');assert.equal(await page.locator('#playback').isVisible(),true);
 await page.locator('#progress').fill('620');await page.locator('#progress').dispatchEvent('input');
 assert.match(await page.locator('#motion-status').textContent(),/XYZ/);
 await page.screenshot({path:'media/03-nc-preview.png',fullPage:true});
 await page.click('#play');await page.waitForFunction(()=>Number(document.querySelector('#progress').value)>650);await page.click('#play');
 await page.click('#top');
 assert.equal(await page.evaluate(()=>{const a=window.pixcam.project([0,0,0]),b=window.pixcam.project([0,76,0]),c=window.pixcam.project([0,0,20]);return Math.abs(b[1]-a[1])>100&&Math.abs(c[1]-a[1])<0.001;}),true);
 await page.click('#iso');
 const before=await page.locator('#view').evaluate(c=>c.toDataURL());await page.locator('#view').focus();await page.keyboard.press('ArrowRight');const after=await page.locator('#view').evaluate(c=>c.toDataURL());assert.notEqual(before,after);
 for(const name of ['features.json','features.csv','feature-map.svg','feature-map.dxf']){const promise=page.waitForEvent('download');await page.locator(`a[download][href="${name}"]`).click();const download=await promise;assert.equal(download.suggestedFilename(),name);const target=path.join('test-results',name);await download.saveAs(target);assert.deepEqual(fs.readFileSync(target),fs.readFileSync(path.join('dist',name)));}
 const pngWait=page.waitForEvent('download');await page.click('#png');const png=await pngWait;assert.equal(png.suggestedFilename(),'pixcam-aurora.png');await png.saveAs('test-results/canvas.png');
 await page.locator('#limits').evaluate(el=>el.open=true);assert.match(await page.locator('#limitations').textContent(),/No verified machining simulation/);
 await page.setViewportSize({width:390,height:844});assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);await page.locator('#map-tab').scrollIntoViewIfNeeded();await page.click('#map-tab');assert.equal(await page.locator('#view').isVisible(),true);
 await page.goto(pathToFileURL(path.resolve('dist/feature-map.svg')).href);assert.equal(await page.locator('parsererror').count(),0);assert.equal(await page.locator('polyline').count(),9);
 assert.deepEqual(errors,[]);assert.deepEqual(network,[]);
 console.log('PASS: filters, details, groups, playback, keyboard orbit, four downloads, PNG, limits, mobile, SVG, no network or JS errors.');
}finally{await browser.close();}
