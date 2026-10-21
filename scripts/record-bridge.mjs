// Real browser frames from our synthetic-only GUI connection; never captures the desktop.
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import {pathToFileURL} from 'node:url';
const module=process.argv[2]||'playwright';
const {chromium}=await import(module.startsWith('/')?pathToFileURL(module).href:module);
if(!process.env.SOLIDCRAFT_TEST_CONNECT)throw Error('Run inside live-gui-test.py --record for an isolated synthetic GUI');
const adapter=spawn('target/debug/pixcam-showcase',['bridge','--connect',process.env.SOLIDCRAFT_TEST_CONNECT],{stdio:['ignore','pipe','ignore']});
let browser,ffmpeg;
try {
  const url=await new Promise((resolve,reject)=>{adapter.stdout.once('data',b=>resolve(b.toString().match(/http:\/\/127\.0\.0\.1:\d+\//)[0]));adapter.once('error',reject);});
  browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--disable-gpu']});
  const page=await browser.newPage({viewport:{width:1600,height:1200}});await page.goto(url);
  await page.locator('#bridge-product').scrollIntoViewIfNeeded();
  const caption=async text=>page.evaluate(text=>{const c=document.getElementById('tour-caption');c.hidden=false;c.textContent=text;},text);
  ffmpeg=spawn('ffmpeg',['-y','-hide_banner','-loglevel','error','-f','image2pipe','-framerate','10','-vcodec','mjpeg','-i','pipe:0','-c:v','libx264','-pix_fmt','yuv420p','-crf','22','-movflags','+faststart','-an','media/pixcam-solidcraft-bridge-36s.mp4'],{stdio:['pipe','ignore','inherit']});
  const finished=once(ffmpeg,'close');
  const steps=new Map([
    [0,async()=>caption('SolidCraft 0.17.0: our synthetic document is open in an isolated GUI.')],
    [35,async()=>{await page.click('#bridge-connected-button');await caption('pixCAM connects locally through the read-only Rust MCP adapter.');}],
    [65,async()=>{await page.click('#bridge-refresh');await page.waitForFunction(()=>document.querySelector('#bridge-status').textContent.includes('/ refreshed'),{},{timeout:65000});await caption('Actual CAD geometry: 100 × 70 × 30 mm, 24 surfaces, five cylindrical segments.');}],
    [105,async()=>{await page.click('[data-feature="CY02"]');await caption('The blind Ø10 segment measures 12 mm. Geometry and uncertainty remain explicit.');}],
    [145,async()=>{await page.selectOption('#bridge-filter','coaxial');await page.click('[data-feature="CY03"]');await caption('Coaxial Ø12 / Ø6 surfaces suggest a counterbore; the operation stays inferred.');}],
    [190,async()=>{await page.click('[data-feature="CY05"]');await page.selectOption('#bridge-setup','1');await caption('Rotate the setup. The side axis aligns with Machine Z; local XYZ updates.');}],
    [240,async()=>{await page.selectOption('#bridge-setup','3');await page.click('#bridge-map');await caption('3+2 geometry and surface normals support planning, without certified machine kinematics.');}],
    [285,async()=>{await page.locator('.bridge-bottom').scrollIntoViewIfNeeded();await caption('Export the current map and evidence: JSON, CSV, SVG, snapshot and PNG.');}],
    [330,async()=>caption('From SolidCraft CAD to Manufacturing Intelligence / pixCAM v0.3')],
  ]);
  const start=performance.now();
  for(let frame=0;frame<360;frame++){
    if(steps.has(frame))await steps.get(frame)();
    const bytes=await page.screenshot({type:'jpeg',quality:85});if(!ffmpeg.stdin.write(bytes))await once(ffmpeg.stdin,'drain');
    const delay=(frame+1)*100-(performance.now()-start);if(delay>0)await new Promise(resolve=>setTimeout(resolve,delay));
  }
  ffmpeg.stdin.end();const [code]=await finished;if(code!==0)throw Error('ffmpeg failed');
  console.log('Recorded 360 actual browser frames / 36 seconds / isolated SolidCraft GUI MCP connection.');
}finally{if(ffmpeg&&ffmpeg.exitCode===null)ffmpeg.kill();if(browser)await browser.close();adapter.kill();}
