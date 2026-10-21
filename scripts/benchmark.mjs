import {spawnSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {performance} from 'node:perf_hooks';

const runs=Number(process.argv[2]||100);
if(!Number.isSafeInteger(runs)||runs<5||runs>10000)throw new Error('runs must be an integer from 5 to 10000');
const binary=path.resolve('target/release/pixcam-showcase');
if(!fs.existsSync(binary))throw new Error('Build release binary first: cargo build --release');
const output=fs.mkdtempSync(path.join(os.tmpdir(),'pixcam-v02-benchmark-'));
const samples=[];
try{
  for(let index=0;index<runs;index++){
    const start=performance.now();
    const result=spawnSync(binary,['build',output],{stdio:'ignore'});
    if(result.status!==0)throw new Error(`build failed at run ${index+1}`);
    samples.push(performance.now()-start);
  }
  samples.sort((a,b)=>a-b);
  const percentile=p=>samples[Math.min(samples.length-1,Math.ceil(samples.length*p)-1)];
  const bytes=fs.readdirSync(output).reduce((sum,name)=>sum+fs.statSync(path.join(output,name)).size,0);
  console.log(JSON.stringify({runs,median_ms:Number(percentile(.5).toFixed(3)),p95_ms:Number(percentile(.95).toFixed(3)),min_ms:Number(samples[0].toFixed(3)),max_ms:Number(samples.at(-1).toFixed(3)),output_files:fs.readdirSync(output).length,output_bytes:bytes},null,2));
}finally{
  fs.rmSync(output,{recursive:true,force:true});
}
