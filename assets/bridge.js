// Offline snapshot recognition mirrors the bounded Rust registry; parity is tested in Chromium.
(() => {
  'use strict';
  const el=id=>document.getElementById(id),demoReport=JSON.parse(el('bridge-demo').textContent);
  const session='__LOCAL_SESSION__',source='__LOCAL_SOURCE__';
  const hosted=location.protocol==='http:'&&location.hostname==='127.0.0.1'&&!session.startsWith('__');
  const finite3=p=>Array.isArray(p)&&p.length===3&&p.every(Number.isFinite),sub=(a,b)=>a.map((v,i)=>v-b[i]),dot=(a,b)=>a.reduce((v,x,i)=>v+x*b[i],0),cross=(a,b)=>[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]],norm=a=>Math.hypot(...a),tol=1e-5;
  const canonical=a=>{const sign=a.find(v=>Math.abs(v)>tol)<0?-1:1;return a.map(v=>Math.abs(v)<1e-12?0:v*sign);};
  const sameAxis=(a,p,b,q)=>norm(cross(a,b))<tol&&norm(cross(sub(q,p),a))<tol;
  function localFrame(axis,origin){let x=cross(Math.abs(axis[0])<.8?[1,0,0]:[0,1,0],axis);x=x.map(v=>v/norm(x));const y=cross(axis,x);return [[x[0],y[0],axis[0],origin[0]],[x[1],y[1],axis[1],origin[1]],[x[2],y[2],axis[2],origin[2]],[0,0,0,1]].map(row=>row.map(v=>v===0?0:v));}
  const fail=()=>{throw Error('Incomplete or invalid SolidCraft 0.17.0 / mm geometry snapshot');};
  function analyze(input){
    // Recompute reports from their source snapshot; never trust imported inferred fields.
    const s=structuredClone(input.snapshot||input),d=s.inspect_design;
    if(s.schema_version!==1||s.solidcraft_version!=='0.17.0'||!d||(d.units||'mm')!=='mm'||!Array.isArray(d.bodies)||!Array.isArray(s.body_topology)||d.bodies.length>256)fail();
    const names=new Set();
    for(const b of d.bodies){if(typeof b.name!=='string'||!b.name||names.has(b.name)||!finite3(b.bbox?.min)||!finite3(b.bbox?.max)||b.bbox.min.some((v,i)=>v>b.bbox.max[i]))fail();names.add(b.name);}
    const seen=new Set(),features=[],surfaces=[];
    for(const t of s.body_topology){
      if(!names.has(t.body)||seen.has(t.body)||!Array.isArray(t.faces)||t.faces.length>100000)fail();seen.add(t.body);
      t.edges??=[];if(!Array.isArray(t.edges)||t.edges.length>200000)fail();
      const edgeIds=new Set(),faceIds=new Set(),faceNames=new Set();
      for(const e of t.edges){if(!Number.isInteger(e.index)||e.index<0||edgeIds.has(e.index)||typeof e.name!=='string'||![e.start,e.mid,e.end].every(finite3)||!Number.isFinite(e.length)||e.length<0)fail();edgeIds.add(e.index);}
      for(const f of t.faces){
        const normal=f.plane_normal??f.normal??null,c=f.cylinder;
        if(!Number.isInteger(f.index)||f.index<0||faceIds.has(f.index)||typeof f.name!=='string'||!f.name||faceNames.has(f.name)||![f.centroid,f.point].every(finite3)||!Number.isFinite(f.area)||f.area<0||(normal&&(!finite3(normal)||Math.abs(norm(normal)-1)>tol)))fail();
        faceIds.add(f.index);faceNames.add(f.name);
        surfaces.push({body:t.body,face_index:f.index,persistent_name:f.name,area_mm2:f.area,centroid_mm:f.centroid,normal,kind:c?'cylindrical':normal?'planar':'unknown',evidence_level:'VERIFIED_GEOMETRY'});
        if(!c)continue;
        if(!finite3(c.origin)||!finite3(c.axis)||Math.abs(norm(c.axis)-1)>1e-8||!Number.isFinite(c.radius)||c.radius<=0||Math.abs(norm(cross(sub(f.point,c.origin),c.axis))-c.radius)>1e-3)fail();
        const axis=canonical(c.axis),points=t.edges.filter(e=>e.name.split('|').some(part=>part.split('~')[0]===f.name)).flatMap(e=>[e.start,e.mid,e.end]);
        let range=null;if(points.length){let lo=Infinity,hi=-Infinity;for(const p of points){const v=dot(p,axis);lo=Math.min(lo,v);hi=Math.max(hi,v);}range=[lo,hi];}
        const origin=sub(c.origin,axis.map(v=>v*dot(c.origin,axis)));
        const merge=features.find(o=>o.body===t.body&&Math.abs(o.diameter_mm-c.radius*2)<tol&&sameAxis(o.axis,o.axis_origin_mm,axis,origin)&&o.axial_range_mm&&range&&o.axial_range_mm[0]<=range[1]+tol&&range[0]<=o.axial_range_mm[1]+tol);
        if(merge){merge.face_indices.push(f.index);merge.face_names.push(f.name);merge.axial_range_mm=[Math.min(merge.axial_range_mm[0],range[0]),Math.max(merge.axial_range_mm[1],range[1])];merge.boundary_span_mm=merge.axial_range_mm[1]-merge.axial_range_mm[0];continue;}
        const bore=/:bore:|:cbore:/.test(f.name),local=range?origin.map((v,i)=>v+axis[i]*range[0]):c.origin;
        features.push({id:'CY'+String(features.length+1).padStart(2,'0'),body:t.body,face_indices:[f.index],face_names:[f.name],evidence_level:'VERIFIED_GEOMETRY',diameter_mm:c.radius*2,axis,axis_origin_mm:origin,axial_range_mm:range,boundary_span_mm:range?range[1]-range[0]:null,cad_from_local:localFrame(axis,local),inferred_feature:bore?'cylindrical_hole_candidate':'cylindrical_surface_unknown',inference_level:bore?'INFERRED_FEATURE':'UNKNOWN',unknown:['Closed 360-degree hole / inward material side','Blind versus through termination (not certified by topology alone)','Threads, tolerances, process and tool'],evidence:'Radius/axis from CAD cylinder; span is projection of named adjacent edge endpoints/midpoints, not an invented hole depth. Hole hypothesis uses CAD persistent naming only.'});
      }
    }
    const groups={},coaxial_groups=[],potential_operations=[];
    for(const f of features){for(const key of [`diameter / ${f.diameter_mm.toFixed(5)} mm`,`parallel axis / ${f.axis.map(v=>v.toFixed(5)).join(',')}`]) (groups[key]??=[]).push(f.id);}
    for(let i=0;i<features.length;i++){const f=features[i];if(coaxial_groups.some(g=>g.includes(f.id)))continue;const g=features.slice(i).filter(o=>o.body===f.body&&sameAxis(f.axis,f.axis_origin_mm,o.axis,o.axis_origin_mm)).map(o=>o.id);if(g.length>1)coaxial_groups.push(g);}
    for(const g of coaxial_groups){const members=features.filter(f=>g.includes(f.id));if(members.some(a=>members.some(b=>Math.abs(a.diameter_mm-b.diameter_mm)>tol&&a.axial_range_mm&&b.axial_range_mm&&(Math.abs(a.axial_range_mm[1]-b.axial_range_mm[0])<tol||Math.abs(b.axial_range_mm[1]-a.axial_range_mm[0])<tol))))potential_operations.push({level:'INFERRED_FEATURE',kind:'stepped_diameters_possible_counterbore',features:g,requires_review:'Material side, complete circumference, access and tooling'});}
    for(const [direction,axis] of [['+Z',[0,0,1]],['-Z',[0,0,-1]],['+X',[1,0,0]],['-X',[-1,0,0]],['+Y',[0,1,0]],['-Y',[0,-1,0]]]){const ids=features.filter(f=>norm(cross(f.axis,axis))<tol).map(f=>f.id);if(ids.length)potential_operations.push({level:'INFERRED_FEATURE',kind:'axis_aligned_setup_candidates',tool_direction:direction,features:ids,requires_review:'Direction sign, access, collisions, material and operation must be verified'});}
    const limitations=[...demoReport.limitations.filter(l=>!l.startsWith('Incomplete topology:'))];if(s.body_topology.length!==d.bodies.length)limitations.push('Incomplete topology: some bodies have no surface data; analysis is partial.');
    // Drop arbitrary metadata, file paths and unknown envelope properties on export.
    const snapshot={schema_version:1,solidcraft_version:'0.17.0',inspect_design:{name:d.name||'',units:'mm',revision:d.revision??null,bodies:d.bodies.map(b=>({name:b.name,bbox:b.bbox})),timeline:[]},body_topology:s.body_topology.map(t=>({body:t.body,faces:t.faces.map(f=>({index:f.index,name:f.name,area:f.area,centroid:f.centroid,point:f.point,plane_normal:f.plane_normal??f.normal??null,cylinder:f.cylinder?{origin:f.cylinder.origin,axis:f.cylinder.axis,radius:f.cylinder.radius}:null})),edges:t.edges.map(e=>({index:e.index,name:e.name,start:e.start,mid:e.mid,end:e.end,length:e.length}))})),measurements:[]};
    return {schema_version:1,product:'pixCAM 0.3.0',document:d.name||'',units:'mm',read_only:true,bodies:snapshot.inspect_design.bodies,features,surfaces,groups,coaxial_groups,potential_operations,limitations,snapshot};
  }
  let report=demoReport,selected=report.features[0]?.id,dataMode='DEMO',surfaceMap=false,yaw=-.68,pitch=.72,zoom=1,rotation=[0,0,0],animation=null,busy=false,stale=false;
  const canvas=el('bridge-canvas'),ctx=canvas.getContext('2d');
  const current=()=>report.features.find(f=>f.id===selected),num=v=>v==null?'UNKNOWN':v.toFixed(3),vfmt=p=>p.map(num).join(' / ');
  const bounds=()=>{const bs=report.bodies.map(b=>b.bbox);return {min:[0,1,2].map(i=>bs.length?Math.min(...bs.map(b=>b.min[i])):0),max:[0,1,2].map(i=>bs.length?Math.max(...bs.map(b=>b.max[i])):100)};};
  const centre=()=>{const b=bounds();return b.min.map((v,i)=>(v+b.max[i])/2);};
  const cadMatrix=()=>{const c=centre();return mul(mul(mul(mul(tr(c),rz(rotation[2])),ry(rotation[1])),rx(rotation[0])),tr(c.map(v=>-v)));};
  function projected(p){p=point(cadMatrix(),p);const c=centre(),b=bounds(),size=Math.max(...sub(b.max,b.min),1),x=p[0]-c[0],y=p[1]-c[1],z=p[2]-c[2],a=x*Math.cos(yaw)-y*Math.sin(yaw),bb=x*Math.sin(yaw)+y*Math.cos(yaw),scale=Math.min(canvas.clientWidth/(size*1.45),canvas.clientHeight/(size*1.1))*zoom;return [canvas.clientWidth/2+a*scale,canvas.clientHeight/2+(bb*Math.cos(pitch)-z*Math.sin(pitch))*scale];}
  function line(points,color,width=1,dash=[]){if(!points.length)return;ctx.beginPath();points.map(projected).forEach(([x,y],i)=>i?ctx.lineTo(x,y):ctx.moveTo(x,y));ctx.strokeStyle=color;ctx.lineWidth=width;ctx.setLineDash(dash);ctx.stroke();ctx.setLineDash([]);}
  function label(p,text,color){const q=projected(p);ctx.fillStyle=color;ctx.font='10px ui-monospace';ctx.fillText(text,q[0]+6,q[1]-6);}
  function ring(f,t){const m=f.cad_from_local,base=point(m,[0,0,0]),z=t-dot(base,f.axis),r=f.diameter_mm/2;return Array.from({length:65},(_,i)=>point(m,[r*Math.cos(i*Math.PI/32),r*Math.sin(i*Math.PI/32),z]));}
  function draw(){
    const w=canvas.clientWidth,h=canvas.clientHeight;ctx.clearRect(0,0,w,h);
    for(const t of report.snapshot.body_topology)for(const e of t.edges)line([e.start,e.mid,e.end],'#527886',1);
    if(surfaceMap)for(const s of report.surfaces){const p=s.centroid_mm;label(p,String(s.face_index),s.kind==='planar'?'#62a9ff':'#55e6d2');if(s.normal)line([p,p.map((v,i)=>v+s.normal[i]*5)],'#62a9ff',1);}
    for(const f of report.features){const color=f.id===selected?'#ffc66d':'#55e6d2',r=f.axial_range_mm;
      if(r){for(const t of r)line(ring(f,t),color,f.id===selected?2:1);const ends=r.map(t=>f.axis_origin_mm.map((v,i)=>v+f.axis[i]*t));line(ends,color,1,[4,4]);label(ends[1],f.id,color);}
      else label(point(f.cad_from_local,[0,0,0]),f.id+' / span unknown',color);
    }
    const f=current();if(f){const m=f.cad_from_local,o=point(m,[0,0,0]);for(const [p,color,name]of [[[12,0,0],'#ff7088','x'],[[0,12,0],'#55e6d2','y'],[[0,0,12],'#62a9ff','z']]){const end=point(m,p);line([o,end],color,2);label(end,name,color);}}
    el('bridge-canvas-label').textContent=(surfaceMap?'SURFACE NORMAL MAP':'CAD TOPOLOGY + ANALYTIC CYLINDERS')+' / '+dataMode+(stale?' / STALE':'');
  }
  function inspector(){
    const f=current(),dl=el('bridge-details');dl.replaceChildren();
    if(!f){el('bridge-feature-title').textContent='No cylindrical surfaces';el('bridge-matrix').replaceChildren();el('bridge-inference').textContent='UNKNOWN · no cylindrical geometry';el('bridge-unknown').replaceChildren();return;}
    el('bridge-feature-title').textContent=`${f.id} / Ø${num(f.diameter_mm)}`;
    const origin=point(f.cad_from_local,[0,0,0]),machine=point(cadMatrix(),origin),axis=vector(cadMatrix(),f.axis),local=point(inv(f.cad_from_local),origin);
    const rows=[['Body / CAD faces',`${f.body} / ${f.face_indices.join(', ')}`],['Axis origin / CAD XYZ',vfmt(origin)+' mm'],['Global Machine XYZ',vfmt(machine)+' mm'],['Feature local XYZ',vfmt(local)+' mm'],['Diameter / CAD',num(f.diameter_mm)+' mm'],['Axis / CAD',vfmt(f.axis)],['Axis / Machine',vfmt(axis)],['Available boundary span',num(f.boundary_span_mm)+(f.boundary_span_mm==null?'':' mm')],['Axial interval / CAD',f.axial_range_mm?f.axial_range_mm.map(num).join(' … ')+' mm':'UNKNOWN'],['Machine Z alignment',Math.abs(axis[2])>1-1e-6?'Parallel / access requires review':'Requires another orientation']];
    for(const [name,value]of rows){const dt=document.createElement('dt'),dd=document.createElement('dd');dt.textContent=name;dd.textContent=value;dl.append(dt,dd);}
    el('bridge-matrix').innerHTML=matrixTable(f.cad_from_local);
    el('bridge-inference').textContent=f.inference_level+' · '+f.inferred_feature.replaceAll('_',' ');
    el('bridge-unknown').replaceChildren();for(const u of f.unknown){const li=document.createElement('li');li.textContent=u;el('bridge-unknown').append(li);}
  }
  function featureList(){const root=el('bridge-features');root.replaceChildren();for(const f of report.features){const b=document.createElement('button');b.className='bridge-feature'+(f.id===selected?' selected':'');b.dataset.feature=f.id;const id=document.createElement('b'),diam=document.createElement('span'),hint=document.createElement('small');id.textContent=f.id;diam.textContent='Ø'+num(f.diameter_mm);hint.textContent=`Axis ${vfmt(f.axis)} · ${num(f.boundary_span_mm)} mm`;b.append(id,diam,hint);b.onclick=()=>{selected=f.id;featureList();inspector();draw();};root.append(b);}}
  function groupList(){const filter=el('bridge-filter').value,root=el('bridge-groups');root.replaceChildren();let entries=Object.entries(report.groups).filter(([key])=>filter==='diameter'?key.startsWith('diameter'):filter==='orientation'?key.startsWith('parallel'):false);if(filter==='coaxial')entries=report.coaxial_groups.map((g,i)=>['coaxial '+(i+1),g]);for(const [key,ids]of entries){const div=document.createElement('div');div.textContent=key+' → '+ids.join(', ');root.append(div);}}
  function render(){
    selected=report.features.some(f=>f.id===selected)?selected:report.features[0]?.id;
    el('bridge-document').textContent=report.document||'Unnamed';const b=bounds();el('bridge-dimensions').textContent=report.bodies.length?sub(b.max,b.min).map(v=>num(v)).join(' × '):'No bodies';
    el('bridge-cylinder-count').textContent=String(report.features.length).padStart(2,'0');el('bridge-surface-count').textContent=report.surfaces.length;
    featureList();groupList();inspector();draw();
    el('bridge-operations').replaceChildren();for(const op of report.potential_operations){const div=document.createElement('div');div.className='bridge-operation';const b=document.createElement('b');b.textContent=(op.tool_direction?`Tool ${op.tool_direction} / parallel axis candidates`:'Stepped diameters / possible counterbore');div.append(b,document.createElement('br'),document.createTextNode(op.features.join(' · ')+' / INFERRED_FEATURE'));el('bridge-operations').append(div);}
    el('bridge-limits').replaceChildren();for(const l of report.limitations){const li=document.createElement('li');li.textContent=l;el('bridge-limits').append(li);}
  }
  function setDataMode(mode){if(busy)return;dataMode=mode;for(const name of ['demo','import','connected'])el('bridge-'+name+'-button').classList.toggle('active',mode===name.toUpperCase());el('bridge-file-label').hidden=mode!=='IMPORT';el('bridge-refresh').hidden=mode!=='CONNECTED';
    if(mode==='DEMO'){report=demoReport;stale=false;el('bridge-status').textContent='DEMO · offline / actual synthetic CAD capture';el('bridge-source').textContent='Synthetic specimen · SolidCraft 0.17.0 → read-only MCP → pixCAM registry · captured geometry, not a live session';render();}
    else if(mode==='IMPORT'){el('bridge-status').textContent='IMPORT · choose a snapshot / current display retained';}
    else {el('bridge-status').textContent=hosted?'CONNECTED · ready / '+source:'CONNECTED · adapter required';el('bridge-source').textContent=hosted?'Local Rust adapter → '+source+' · refresh explicitly to read the current document':'Run pixcam-showcase bridge --connect 127.0.0.1:PORT and open the localhost URL printed by the adapter. For an isolated test use --in fixtures/bridge-part.solidcraft.';}
  }
  async function api(path,body){const response=await fetch(path,{method:'POST',headers:{'Content-Type':'application/json','X-Pixcam-Session':session},body:JSON.stringify(body),signal:AbortSignal.timeout(65000)});const data=await response.json();if(!response.ok)throw Error(data.error||'Adapter unavailable');return data;}
  el('bridge-file').onchange=async event=>{const file=event.target.files[0];if(!file||busy)return;busy=true;try{if(file.size>16*1024*1024)throw Error('Snapshot exceeds 16 MiB');const input=JSON.parse(await file.text());report=hosted?await api('/api/analyze',input.snapshot||input):analyze(input);stale=false;el('bridge-status').textContent='IMPORT · analyzed locally / '+report.document;el('bridge-source').textContent='Imported geometry snapshot · SolidCraft '+report.snapshot.solidcraft_version+' · '+report.bodies.length+' bodies · no connection to CAD';render();}catch(error){stale=true;el('bridge-status').textContent='IMPORT failed · previous data retained: '+error.message;draw();}finally{busy=false;}};
  el('bridge-refresh').onclick=async()=>{if(!hosted){el('bridge-status').textContent='CONNECTED unavailable · open the Rust adapter URL';return;}if(busy)return;busy=true;el('bridge-refresh').disabled=true;el('bridge-status').textContent='CONNECTED · reading CAD geometry…';try{report=await api('/api/refresh',{});stale=false;el('bridge-status').textContent='CONNECTED · '+source+' / refreshed '+new Date().toLocaleTimeString('en-GB');render();}catch(error){stale=true;el('bridge-status').textContent='DISCONNECTED · stale data retained / '+error.message;draw();}finally{busy=false;el('bridge-refresh').disabled=false;}};
  for(const mode of ['demo','import','connected'])el('bridge-'+mode+'-button').onclick=()=>setDataMode(mode.toUpperCase());
  el('bridge-filter').onchange=groupList;
  el('bridge-setup').onchange=()=>{const to=[[0,0,0],[0,90,0],[-90,0,0],[30,-35,20]][+el('bridge-setup').value];animation={from:[...rotation],to,start:performance.now()};requestAnimationFrame(function animate(t){if(!animation)return;const progress=Math.min(1,(t-animation.start)/600),ease=progress*progress*(3-2*progress);rotation=animation.from.map((v,i)=>v+(animation.to[i]-v)*ease);draw();inspector();if(progress<1)requestAnimationFrame(animate);else animation=null;});};
  el('bridge-iso').onclick=()=>{yaw=-.68;pitch=.72;zoom=1;draw();};el('bridge-top').onclick=()=>{yaw=0;pitch=0;zoom=1;draw();};el('bridge-map').onclick=()=>{surfaceMap=!surfaceMap;el('bridge-map').classList.toggle('active',surfaceMap);draw();};
  let drag=null;canvas.onpointerdown=e=>{drag=[e.clientX,e.clientY];canvas.setPointerCapture(e.pointerId);};canvas.onpointermove=e=>{if(!drag)return;yaw+=(e.clientX-drag[0])*.009;pitch=Math.max(-1.5,Math.min(1.5,pitch+(e.clientY-drag[1])*.009));drag=[e.clientX,e.clientY];draw();};canvas.onpointerup=canvas.onpointercancel=()=>drag=null;
  canvas.addEventListener('wheel',e=>{e.preventDefault();zoom=Math.max(.3,Math.min(4,zoom*Math.exp(-e.deltaY*.001)));draw();},{passive:false});canvas.onkeydown=e=>{if(e.key.startsWith('Arrow')){e.preventDefault();yaw+=e.key==='ArrowRight'?.1:e.key==='ArrowLeft'?-.1:0;pitch+=e.key==='ArrowDown'?.1:e.key==='ArrowUp'?-.1:0;draw();}};
  new ResizeObserver(()=>{const ratio=devicePixelRatio||1;canvas.width=Math.max(1,canvas.clientWidth*ratio);canvas.height=Math.max(1,canvas.clientHeight*ratio);ctx.setTransform(ratio,0,0,ratio,0,0);draw();}).observe(canvas);
  const download=(name,text,type='application/json')=>{const url=URL.createObjectURL(new Blob([text],{type})),a=document.createElement('a');a.href=url;a.download=name;a.click();setTimeout(()=>URL.revokeObjectURL(url),1000);};
  const csv=()=>{const keys=['id','body','evidence_level','inference_level','diameter_mm','axis_x','axis_y','axis_z','axis_origin_x','axis_origin_y','axis_origin_z','boundary_span_mm','hypothesis'];return keys.join(',')+'\r\n'+report.features.map(f=>[f.id,f.body,f.evidence_level,f.inference_level,f.diameter_mm,...f.axis,...f.axis_origin_mm,f.boundary_span_mm??'',f.inferred_feature].map(v=>'"'+String(v).replaceAll('"','""')+'"').join(',')).join('\r\n')+'\r\n';};
  const xml=s=>String(s).replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('"','&quot;');
  function svg(){const b=bounds(),size=Math.max(...sub(b.max,b.min),1);let result='<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 420"><rect width="1200" height="420" fill="#09151d"/><text x="20" y="25" fill="#55e6d2" font-size="16">pixCAM / CAD surface map / '+xml(report.document)+'</text>';
    for(const [index,axes]of [[0,[0,1]],[1,[0,2]],[2,[1,2]]]){const project=p=>[index*400+35+(p[axes[0]]-b.min[axes[0]])/size*330,350-(p[axes[1]]-b.min[axes[1]])/size*280];result+=`<text x="${index*400+35}" y="60" fill="#ecf7f7">${['XY','XZ','YZ'][index]} / mm</text>`;for(const t of report.snapshot.body_topology)for(const e of t.edges)result+='<polyline points="'+[e.start,e.mid,e.end].map(p=>project(p).join(',')).join(' ')+'" stroke="#527886" fill="none"/>';for(const f of report.features){if(!f.axial_range_mm)continue;for(const range of f.axial_range_mm)result+='<polyline points="'+ring(f,range).map(p=>project(p).join(',')).join(' ')+'" stroke="#55e6d2" fill="none"/>';const p=project(point(f.cad_from_local,[0,0,0]));result+=`<text x="${p[0]}" y="${p[1]-5}" fill="#ffc66d" font-size="10">${f.id}</text>`;}}
    return result+'<text x="20" y="400" fill="#8fa7ad" font-size="12">Projected boundary samples + analytic cylinders. Planning aid / incomplete geometry stays unknown.</text></svg>';}
  el('bridge-json').onclick=()=>download('pixcam-report.json',JSON.stringify(report,null,2));el('bridge-csv').onclick=()=>download('pixcam-registry.csv',csv(),'text/csv');el('bridge-snapshot').onclick=()=>download('solidcraft-snapshot.json',JSON.stringify(report.snapshot,null,2));el('bridge-svg').onclick=()=>download('pixcam-surface-map.svg',svg(),'image/svg+xml');el('bridge-png').onclick=()=>{const a=document.createElement('a');a.href=canvas.toDataURL();a.download='pixcam-cad-view.png';a.click();};
  el('legacy-toggle').onclick=()=>{const lab=el('legacy-lab');lab.hidden=!lab.hidden;el('legacy-toggle').setAttribute('aria-expanded',String(!lab.hidden));};
  document.querySelector('.version').textContent='SOLIDCRAFT BRIDGE · v0.3';document.querySelector('.safe').textContent='● Local geometry · read-only';
  window.pixcamBridge={analyze,get report(){return report;},get dataMode(){return dataMode;},get stale(){return stale;},csv,svg};render();
})();
