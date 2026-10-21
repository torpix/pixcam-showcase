// Records only the synthetic browser viewport, never the desktop or browser chrome.
import {pathToFileURL} from 'node:url';
import path from 'node:path';
import fs from 'node:fs';
import {spawn} from 'node:child_process';
import {once} from 'node:events';

const module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
fs.mkdirSync('media',{recursive:true});
const browser=await chromium.launch({executablePath:process.env.CHROMIUM||'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
const page=await browser.newPage({viewport:{width:1440,height:1280},deviceScaleFactor:1});
const output='media/pixcam-coordinate-inspector-72s.mp4';
const ffmpeg=spawn('ffmpeg',['-y','-hide_banner','-loglevel','error','-f','image2pipe','-framerate','10','-vcodec','mjpeg','-i','pipe:0','-c:v','libx264','-pix_fmt','yuv420p','-crf','22','-movflags','+faststart','-an',output],{stdio:['pipe','ignore','inherit']});
const finished=once(ffmpeg,'close');
const caption=async text=>page.evaluate(text=>{const el=document.getElementById('tour-caption');el.hidden=false;el.textContent=text;},text);
const steps=new Map([
  [0,async()=>caption('pixCAM v0.2 — prove the setup, prove the coordinates')],
  [60,async()=>{await page.evaluate(()=>window.pixcam.selectSetup(1));await caption('Four synthetic 3+2 orientations animate around the stock centre.');}],
  [125,async()=>{await page.evaluate(()=>window.pixcam.selectSetup(2));await page.selectOption('#frame','Setup');await caption('Machine, Workpiece, Setup, Tool and CAD frames remain explicit.');}],
  [190,async()=>{await page.evaluate(()=>window.pixcam.selectSetup(3));await page.selectOption('#frame','CAD');await caption('The live 4×4 matrix and local/global coordinates update together.');}],
  [265,async()=>{await page.click('[data-id="A04"]');await caption('Select any hole to inspect its position and axis in the chosen frame.');}],
  [325,async()=>{await page.click('#map-tab');await page.click('[data-id="S01"]');await caption('The public Aurora plate contains seven holes, a slot and a pocket.');}],
  [385,async()=>{await page.locator('[data-motion-index="16"]').click();await caption('Every G0, G1, G2 and G3 move has a clickable geometric explanation.');}],
  [450,async()=>{await page.click('#play');await caption('Arcs use deterministic interpolated trajectories; playback follows path distance.');}],
  [525,async()=>{await page.click('#play');await page.locator('[data-motion-index="18"]').click();await caption('Start, end, length, direction, tool axis, frame and feature link stay visible.');}],
  [590,async()=>{await page.locator('.foot-grid').scrollIntoViewIfNeeded();await caption('JSON, CSV, SVG, DXF and the complete HTML work offline.');}],
  [650,async()=>{await page.locator('#limits').evaluate(el=>el.open=true);await caption('No CYCLE800 claim, fake Boolean removal or solved machine kinematics.');}],
  [695,async()=>caption('pixCAM v0.2 — an original read-only companion for SolidCraft')],
]);

try{
  await page.goto(pathToFileURL(path.resolve('dist/index.html')).href);
  const start=performance.now();
  for(let frame=0;frame<720;frame++){
    if(steps.has(frame))await steps.get(frame)();
    const bytes=await page.screenshot({type:'jpeg',quality:83});
    if(!ffmpeg.stdin.write(bytes))await once(ffmpeg.stdin,'drain');
    const delay=(frame+1)*100-(performance.now()-start);
    if(delay>0)await new Promise(resolve=>setTimeout(resolve,delay));
  }
  ffmpeg.stdin.end();
  const [code]=await finished;
  if(code!==0)throw new Error(`ffmpeg exit ${code}`);
  console.log(`Recorded 720 browser frames / 72 seconds, 1440×1280, no desktop capture: ${output}`);
}finally{
  await browser.close();
  if(ffmpeg.exitCode===null)ffmpeg.kill();
}
