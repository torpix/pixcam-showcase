import assert from 'node:assert/strict';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
import fs from 'node:fs';

const module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
const browser=await chromium.launch({executablePath:process.env.CHROMIUM||'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
fs.mkdirSync('media',{recursive:true});
fs.mkdirSync('test-results',{recursive:true});
const page=await browser.newPage({viewport:{width:1600,height:1100},deviceScaleFactor:1});
const errors=[],network=[];
page.on('pageerror',error=>errors.push(error.message));
page.on('request',request=>{if(/^https?:/.test(request.url()))network.push(request.url());});

try{
  await page.goto(pathToFileURL(path.resolve('dist/index.html')).href);
  await page.click('#legacy-toggle');
  assert.equal(await page.locator('.setup').count(),4);
  assert.equal(await page.locator('.feature').count(),9);
  assert.equal(await page.locator('.motion').count(),27);
  assert.equal(await page.locator('#matrix-wrap .matrix tr').count(),4);
  assert.match(await page.locator('#details').textContent(),/Global Machine XYZ/);
  await page.screenshot({path:'media/01-coordinate-inspector.png',fullPage:true});

  await page.evaluate(()=>window.pixcam.selectSetup(3,true));
  assert.equal(await page.evaluate(()=>window.pixcam.setupIndex),3);
  assert.match(await page.locator('#details').textContent(),/30° \/ -35° \/ 20°/);
  assert.match(await page.locator('#details').textContent(),/A 31° · B -47°/);
  const machineMatrix=await page.locator('#matrix-wrap').textContent();
  await page.selectOption('#frame','CAD');
  assert.notEqual(await page.locator('#matrix-wrap').textContent(),machineMatrix);
  await page.selectOption('#frame','Workpiece');
  assert.match(await page.locator('#detail-subtitle').textContent(),/Workpiece/);

  await page.click('#map-tab');
  await page.selectOption('#face','Y_MIN');
  assert.equal(await page.locator('.feature').count(),2);
  await page.click('[data-id="B02"]');
  assert.match(await page.locator('#details').textContent(),/89\.000 \/ 0\.000 \/ 10\.000/);
  await page.selectOption('#face','');
  await page.screenshot({path:'media/02-feature-map.png',fullPage:true});

  await page.locator('[data-motion-index="18"]').click();
  assert.equal(await page.evaluate(()=>window.pixcam.mode),'nc');
  assert.match(await page.locator('#detail-title').textContent(),/G3/);
  assert.match(await page.locator('#details').textContent(),/Interpolated points/);
  assert.match(await page.locator('#notice').textContent(),/Arc shown/);
  assert.match(await page.locator('#details').textContent(),/S01/);
  await page.screenshot({path:'media/03-nc-inspector.png',fullPage:true});
  await page.locator('#progress').fill('620');
  await page.locator('#progress').dispatchEvent('input');
  assert.match(await page.locator('#motion-status').textContent(),/Line/);
  await page.click('#play');
  await page.waitForFunction(()=>Number(document.querySelector('#progress').value)>625);
  await page.click('#play');

  const before=await page.locator('#view').evaluate(canvas=>canvas.toDataURL());
  await page.locator('#view').focus();
  await page.keyboard.press('ArrowRight');
  assert.notEqual(before,await page.locator('#view').evaluate(canvas=>canvas.toDataURL()));
  for(const name of ['features.json','features.csv','feature-map.svg','feature-map.dxf']){
    const promise=page.waitForEvent('download');
    await page.locator(`a[download][href="${name}"]`).click();
    const download=await promise;
    assert.equal(download.suggestedFilename(),name);
    const target=path.join('test-results',name);
    await download.saveAs(target);
    assert.deepEqual(fs.readFileSync(target),fs.readFileSync(path.join('dist',name)));
  }
  const pngWait=page.waitForEvent('download');
  await page.click('#png');
  const png=await pngWait;
  assert.equal(png.suggestedFilename(),'pixcam-coordinate-inspector.png');
  await png.saveAs('test-results/canvas.png');

  assert.equal(await page.locator('#warnings li').count(),4);
  await page.locator('#limits').evaluate(element=>element.open=true);
  assert.match(await page.locator('#limitations').textContent(),/No verified machining simulation/);
  await page.setViewportSize({width:390,height:844});
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true);
  assert.equal(await page.locator('#view').isVisible(),true);
  await page.goto(pathToFileURL(path.resolve('dist/feature-map.svg')).href);
  assert.equal(await page.locator('parsererror').count(),0);
  assert.equal(await page.locator('polyline').count(),9);
  assert.deepEqual(errors,[]);
  assert.deepEqual(network,[]);
  console.log('PASS: four animated setups, five frames, matrices, feature coordinates, 27 clickable motions, interpolated arcs, four downloads, PNG, responsive UI, SVG, no network or JS errors.');
}finally{
  await browser.close();
}
