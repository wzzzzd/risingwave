"use strict";(self.webpackChunk_N_E=self.webpackChunk_N_E||[]).push([[383],{44527:function(r,e,t){var o=t(85893),E=t(40639);t(67294),e.Z=function(r){let{children:e}=r;return(0,o.jsx)(E.xv,{mb:2,textColor:"blue.500",fontWeight:"semibold",lineHeight:"6",children:e})}},34269:function(r,e){let t=new class{async get(r){try{let e=await fetch(r),t=await e.json();return t}catch(e){throw console.error(e),Error("Failed to fetch "+r)}}};e.Z=t},2789:function(r,e,t){t.d(e,{GS:function(){return S},Kn:function(){return u},L3:function(){return o},XE:function(){return l},XK:function(){return workerTypeFromJSON},Yz:function(){return C},cX:function(){return I},lW:function(){return c},qb:function(){return s},u:function(){return workerTypeToJSON},yM:function(){return R}});let o={WORKER_TYPE_UNSPECIFIED:"WORKER_TYPE_UNSPECIFIED",WORKER_TYPE_FRONTEND:"WORKER_TYPE_FRONTEND",WORKER_TYPE_COMPUTE_NODE:"WORKER_TYPE_COMPUTE_NODE",WORKER_TYPE_RISE_CTL:"WORKER_TYPE_RISE_CTL",WORKER_TYPE_COMPACTOR:"WORKER_TYPE_COMPACTOR",WORKER_TYPE_META:"WORKER_TYPE_META",UNRECOGNIZED:"UNRECOGNIZED"};function workerTypeFromJSON(r){switch(r){case 0:case"WORKER_TYPE_UNSPECIFIED":return o.WORKER_TYPE_UNSPECIFIED;case 1:case"WORKER_TYPE_FRONTEND":return o.WORKER_TYPE_FRONTEND;case 2:case"WORKER_TYPE_COMPUTE_NODE":return o.WORKER_TYPE_COMPUTE_NODE;case 3:case"WORKER_TYPE_RISE_CTL":return o.WORKER_TYPE_RISE_CTL;case 4:case"WORKER_TYPE_COMPACTOR":return o.WORKER_TYPE_COMPACTOR;case 5:case"WORKER_TYPE_META":return o.WORKER_TYPE_META;default:return o.UNRECOGNIZED}}function workerTypeToJSON(r){switch(r){case o.WORKER_TYPE_UNSPECIFIED:return"WORKER_TYPE_UNSPECIFIED";case o.WORKER_TYPE_FRONTEND:return"WORKER_TYPE_FRONTEND";case o.WORKER_TYPE_COMPUTE_NODE:return"WORKER_TYPE_COMPUTE_NODE";case o.WORKER_TYPE_RISE_CTL:return"WORKER_TYPE_RISE_CTL";case o.WORKER_TYPE_COMPACTOR:return"WORKER_TYPE_COMPACTOR";case o.WORKER_TYPE_META:return"WORKER_TYPE_META";case o.UNRECOGNIZED:default:return"UNRECOGNIZED"}}let E={DIRECTION_UNSPECIFIED:"DIRECTION_UNSPECIFIED",DIRECTION_ASCENDING:"DIRECTION_ASCENDING",DIRECTION_DESCENDING:"DIRECTION_DESCENDING",UNRECOGNIZED:"UNRECOGNIZED"},n={NULLS_ARE_UNSPECIFIED:"NULLS_ARE_UNSPECIFIED",NULLS_ARE_LARGEST:"NULLS_ARE_LARGEST",NULLS_ARE_SMALLEST:"NULLS_ARE_SMALLEST",UNRECOGNIZED:"UNRECOGNIZED"},a={UNSPECIFIED:"UNSPECIFIED",OK:"OK",UNKNOWN_WORKER:"UNKNOWN_WORKER",UNRECOGNIZED:"UNRECOGNIZED"},i={UNSPECIFIED:"UNSPECIFIED",STARTING:"STARTING",RUNNING:"RUNNING",UNRECOGNIZED:"UNRECOGNIZED"},N={UNSPECIFIED:"UNSPECIFIED",NONE:"NONE",UNRECOGNIZED:"UNRECOGNIZED"},s={fromJSON:r=>({code:isSet(r.code)?function(r){switch(r){case 0:case"UNSPECIFIED":return a.UNSPECIFIED;case 1:case"OK":return a.OK;case 2:case"UNKNOWN_WORKER":return a.UNKNOWN_WORKER;default:return a.UNRECOGNIZED}}(r.code):a.UNSPECIFIED,message:isSet(r.message)?String(r.message):""}),toJSON(r){let e={};return r.code!==a.UNSPECIFIED&&(e.code=function(r){switch(r){case a.UNSPECIFIED:return"UNSPECIFIED";case a.OK:return"OK";case a.UNKNOWN_WORKER:return"UNKNOWN_WORKER";case a.UNRECOGNIZED:default:return"UNRECOGNIZED"}}(r.code)),""!==r.message&&(e.message=r.message),e},create:r=>s.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={code:a.UNSPECIFIED,message:""};return o.code=null!==(e=r.code)&&void 0!==e?e:a.UNSPECIFIED,o.message=null!==(t=r.message)&&void 0!==t?t:"",o}},l={fromJSON:r=>({host:isSet(r.host)?String(r.host):"",port:isSet(r.port)?Number(r.port):0}),toJSON(r){let e={};return""!==r.host&&(e.host=r.host),0!==r.port&&(e.port=Math.round(r.port)),e},create:r=>l.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={host:"",port:0};return o.host=null!==(e=r.host)&&void 0!==e?e:"",o.port=null!==(t=r.port)&&void 0!==t?t:0,o}},u={fromJSON:r=>({id:isSet(r.id)?Number(r.id):0,workerNodeId:isSet(r.workerNodeId)?Number(r.workerNodeId):0}),toJSON(r){let e={};return 0!==r.id&&(e.id=Math.round(r.id)),0!==r.workerNodeId&&(e.workerNodeId=Math.round(r.workerNodeId)),e},create:r=>u.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={id:0,workerNodeId:0};return o.id=null!==(e=r.id)&&void 0!==e?e:0,o.workerNodeId=null!==(t=r.workerNodeId)&&void 0!==t?t:0,o}},I={fromJSON:r=>({id:isSet(r.id)?Number(r.id):0,type:isSet(r.type)?workerTypeFromJSON(r.type):o.WORKER_TYPE_UNSPECIFIED,host:isSet(r.host)?l.fromJSON(r.host):void 0,state:isSet(r.state)?function(r){switch(r){case 0:case"UNSPECIFIED":return i.UNSPECIFIED;case 1:case"STARTING":return i.STARTING;case 2:case"RUNNING":return i.RUNNING;default:return i.UNRECOGNIZED}}(r.state):i.UNSPECIFIED,parallelUnits:Array.isArray(null==r?void 0:r.parallelUnits)?r.parallelUnits.map(r=>u.fromJSON(r)):[],property:isSet(r.property)?d.fromJSON(r.property):void 0,transactionalId:isSet(r.transactionalId)?Number(r.transactionalId):void 0,resource:isSet(r.resource)?S.fromJSON(r.resource):void 0,startedAt:isSet(r.startedAt)?Number(r.startedAt):void 0}),toJSON(r){var e;let t={};return 0!==r.id&&(t.id=Math.round(r.id)),r.type!==o.WORKER_TYPE_UNSPECIFIED&&(t.type=workerTypeToJSON(r.type)),void 0!==r.host&&(t.host=l.toJSON(r.host)),r.state!==i.UNSPECIFIED&&(t.state=function(r){switch(r){case i.UNSPECIFIED:return"UNSPECIFIED";case i.STARTING:return"STARTING";case i.RUNNING:return"RUNNING";case i.UNRECOGNIZED:default:return"UNRECOGNIZED"}}(r.state)),(null===(e=r.parallelUnits)||void 0===e?void 0:e.length)&&(t.parallelUnits=r.parallelUnits.map(r=>u.toJSON(r))),void 0!==r.property&&(t.property=d.toJSON(r.property)),void 0!==r.transactionalId&&(t.transactionalId=Math.round(r.transactionalId)),void 0!==r.resource&&(t.resource=S.toJSON(r.resource)),void 0!==r.startedAt&&(t.startedAt=Math.round(r.startedAt)),t},create:r=>I.fromPartial(null!=r?r:{}),fromPartial(r){var e,t,E,n,a,N;let s={id:0,type:o.WORKER_TYPE_UNSPECIFIED,host:void 0,state:i.UNSPECIFIED,parallelUnits:[],property:void 0,transactionalId:void 0,resource:void 0,startedAt:void 0};return s.id=null!==(t=r.id)&&void 0!==t?t:0,s.type=null!==(E=r.type)&&void 0!==E?E:o.WORKER_TYPE_UNSPECIFIED,s.host=void 0!==r.host&&null!==r.host?l.fromPartial(r.host):void 0,s.state=null!==(n=r.state)&&void 0!==n?n:i.UNSPECIFIED,s.parallelUnits=(null===(e=r.parallelUnits)||void 0===e?void 0:e.map(r=>u.fromPartial(r)))||[],s.property=void 0!==r.property&&null!==r.property?d.fromPartial(r.property):void 0,s.transactionalId=null!==(a=r.transactionalId)&&void 0!==a?a:void 0,s.resource=void 0!==r.resource&&null!==r.resource?S.fromPartial(r.resource):void 0,s.startedAt=null!==(N=r.startedAt)&&void 0!==N?N:void 0,s}},d={fromJSON:r=>({isStreaming:!!isSet(r.isStreaming)&&!!r.isStreaming,isServing:!!isSet(r.isServing)&&!!r.isServing,isUnschedulable:!!isSet(r.isUnschedulable)&&!!r.isUnschedulable}),toJSON(r){let e={};return!0===r.isStreaming&&(e.isStreaming=r.isStreaming),!0===r.isServing&&(e.isServing=r.isServing),!0===r.isUnschedulable&&(e.isUnschedulable=r.isUnschedulable),e},create:r=>d.fromPartial(null!=r?r:{}),fromPartial(r){var e,t,o;let E={isStreaming:!1,isServing:!1,isUnschedulable:!1};return E.isStreaming=null!==(e=r.isStreaming)&&void 0!==e&&e,E.isServing=null!==(t=r.isServing)&&void 0!==t&&t,E.isUnschedulable=null!==(o=r.isUnschedulable)&&void 0!==o&&o,E}},S={fromJSON:r=>({rwVersion:isSet(r.rwVersion)?String(r.rwVersion):"",totalMemoryBytes:isSet(r.totalMemoryBytes)?Number(r.totalMemoryBytes):0,totalCpuCores:isSet(r.totalCpuCores)?Number(r.totalCpuCores):0}),toJSON(r){let e={};return""!==r.rwVersion&&(e.rwVersion=r.rwVersion),0!==r.totalMemoryBytes&&(e.totalMemoryBytes=Math.round(r.totalMemoryBytes)),0!==r.totalCpuCores&&(e.totalCpuCores=Math.round(r.totalCpuCores)),e},create:r=>S.fromPartial(null!=r?r:{}),fromPartial(r){var e,t,o;let E={rwVersion:"",totalMemoryBytes:0,totalCpuCores:0};return E.rwVersion=null!==(e=r.rwVersion)&&void 0!==e?e:"",E.totalMemoryBytes=null!==(t=r.totalMemoryBytes)&&void 0!==t?t:0,E.totalCpuCores=null!==(o=r.totalCpuCores)&&void 0!==o?o:0,E}},c={fromJSON:r=>({compression:isSet(r.compression)?function(r){switch(r){case 0:case"UNSPECIFIED":return N.UNSPECIFIED;case 1:case"NONE":return N.NONE;default:return N.UNRECOGNIZED}}(r.compression):N.UNSPECIFIED,body:isSet(r.body)?function(r){if(_.Buffer)return Uint8Array.from(_.Buffer.from(r,"base64"));{let e=_.atob(r),t=new Uint8Array(e.length);for(let r=0;r<e.length;++r)t[r]=e.charCodeAt(r);return t}}(r.body):new Uint8Array(0)}),toJSON(r){let e={};return r.compression!==N.UNSPECIFIED&&(e.compression=function(r){switch(r){case N.UNSPECIFIED:return"UNSPECIFIED";case N.NONE:return"NONE";case N.UNRECOGNIZED:default:return"UNRECOGNIZED"}}(r.compression)),0!==r.body.length&&(e.body=function(r){if(_.Buffer)return _.Buffer.from(r).toString("base64");{let e=[];return r.forEach(r=>{e.push(String.fromCharCode(r))}),_.btoa(e.join(""))}}(r.body)),e},create:r=>c.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={compression:N.UNSPECIFIED,body:new Uint8Array(0)};return o.compression=null!==(e=r.compression)&&void 0!==e?e:N.UNSPECIFIED,o.body=null!==(t=r.body)&&void 0!==t?t:new Uint8Array(0),o}},R={fromJSON:r=>({originalIndices:Array.isArray(null==r?void 0:r.originalIndices)?r.originalIndices.map(r=>Number(r)):[],data:Array.isArray(null==r?void 0:r.data)?r.data.map(r=>Number(r)):[]}),toJSON(r){var e,t;let o={};return(null===(e=r.originalIndices)||void 0===e?void 0:e.length)&&(o.originalIndices=r.originalIndices.map(r=>Math.round(r))),(null===(t=r.data)||void 0===t?void 0:t.length)&&(o.data=r.data.map(r=>Math.round(r))),o},create:r=>R.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={originalIndices:[],data:[]};return o.originalIndices=(null===(e=r.originalIndices)||void 0===e?void 0:e.map(r=>r))||[],o.data=(null===(t=r.data)||void 0===t?void 0:t.map(r=>r))||[],o}},O={fromJSON:r=>({direction:isSet(r.direction)?function(r){switch(r){case 0:case"DIRECTION_UNSPECIFIED":return E.DIRECTION_UNSPECIFIED;case 1:case"DIRECTION_ASCENDING":return E.DIRECTION_ASCENDING;case 2:case"DIRECTION_DESCENDING":return E.DIRECTION_DESCENDING;default:return E.UNRECOGNIZED}}(r.direction):E.DIRECTION_UNSPECIFIED,nullsAre:isSet(r.nullsAre)?function(r){switch(r){case 0:case"NULLS_ARE_UNSPECIFIED":return n.NULLS_ARE_UNSPECIFIED;case 1:case"NULLS_ARE_LARGEST":return n.NULLS_ARE_LARGEST;case 2:case"NULLS_ARE_SMALLEST":return n.NULLS_ARE_SMALLEST;default:return n.UNRECOGNIZED}}(r.nullsAre):n.NULLS_ARE_UNSPECIFIED}),toJSON(r){let e={};return r.direction!==E.DIRECTION_UNSPECIFIED&&(e.direction=function(r){switch(r){case E.DIRECTION_UNSPECIFIED:return"DIRECTION_UNSPECIFIED";case E.DIRECTION_ASCENDING:return"DIRECTION_ASCENDING";case E.DIRECTION_DESCENDING:return"DIRECTION_DESCENDING";case E.UNRECOGNIZED:default:return"UNRECOGNIZED"}}(r.direction)),r.nullsAre!==n.NULLS_ARE_UNSPECIFIED&&(e.nullsAre=function(r){switch(r){case n.NULLS_ARE_UNSPECIFIED:return"NULLS_ARE_UNSPECIFIED";case n.NULLS_ARE_LARGEST:return"NULLS_ARE_LARGEST";case n.NULLS_ARE_SMALLEST:return"NULLS_ARE_SMALLEST";case n.UNRECOGNIZED:default:return"UNRECOGNIZED"}}(r.nullsAre)),e},create:r=>O.fromPartial(null!=r?r:{}),fromPartial(r){var e,t;let o={direction:E.DIRECTION_UNSPECIFIED,nullsAre:n.NULLS_ARE_UNSPECIFIED};return o.direction=null!==(e=r.direction)&&void 0!==e?e:E.DIRECTION_UNSPECIFIED,o.nullsAre=null!==(t=r.nullsAre)&&void 0!==t?t:n.NULLS_ARE_UNSPECIFIED,o}},C={fromJSON:r=>({columnIndex:isSet(r.columnIndex)?Number(r.columnIndex):0,orderType:isSet(r.orderType)?O.fromJSON(r.orderType):void 0}),toJSON(r){let e={};return 0!==r.columnIndex&&(e.columnIndex=Math.round(r.columnIndex)),void 0!==r.orderType&&(e.orderType=O.toJSON(r.orderType)),e},create:r=>C.fromPartial(null!=r?r:{}),fromPartial(r){var e;let t={columnIndex:0,orderType:void 0};return t.columnIndex=null!==(e=r.columnIndex)&&void 0!==e?e:0,t.orderType=void 0!==r.orderType&&null!==r.orderType?O.fromPartial(r.orderType):void 0,t}},_="undefined"!=typeof globalThis?globalThis:"undefined"!=typeof self?self:window;function isSet(r){return null!=r}}}]);