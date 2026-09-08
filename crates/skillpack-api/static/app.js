const API_BASE = window.location.origin;
const WS_URL = API_BASE.replace(/^http/, 'ws') + '/ws';

let ws = null;

const $ = (sel) => document.querySelector(sel);
const $$ = (sel) => document.querySelectorAll(sel);

/// Generate a W3C traceparent header value.
function genTraceparent() {
    const hex = (n, len) => n.toString(16).padStart(len, '0');
    const traceId = hex(Math.floor(Math.random() * 0xffffffff), 8) + hex(Math.floor(Math.random() * 0xffffffff), 8) + hex(Math.floor(Math.random() * 0xffffffff), 8) + hex(Math.floor(Math.random() * 0xffffffff), 8);
    const spanId = hex(Math.floor(Math.random() * 0xffffffff), 8) + hex(Math.floor(Math.random() * 0xffffffff), 8);
    return `00-${traceId}-${spanId}-01`;
}

function apiHeaders() {
    return { 'traceparent': genTraceparent() };
}

function logEvent(text) {
    const line = document.createElement('div');
    line.className = 'event-line';
    line.textContent = `${new Date().toLocaleTimeString()}  ${text}`;
    $('#event-log').prepend(line);
}

function setStatus(connected) {
    const el = $('#conn-status');
    el.textContent = connected ? 'Connected' : 'Disconnected';
    el.className = `status ${connected ? 'connected' : 'disconnected'}`;
    $('#btn-connect').disabled = connected;
    $('#btn-disconnect').disabled = !connected;
}

async function refreshSkills() {
    try {
        const res = await fetch(`${API_BASE}/api/v1/skills`, { headers: apiHeaders() });
        const data = await res.json();
        const tbody = $('#skills-table tbody');
        tbody.innerHTML = '';
        (data.skills || []).forEach(s => {
            const tr = document.createElement('tr');
            tr.innerHTML = `<td>${s.skill_ref}</td><td>${s.name || ''}</td><td>${s.grade || ''}</td><td>${s.score ?? ''}</td>`;
            tbody.appendChild(tr);
        });
        logEvent(`Fetched ${(data.skills || []).length} skills`);
    } catch (e) {
        logEvent(`Error fetching skills: ${e.message}`);
    }
}

function connectWs() {
    if (ws) return;
    ws = new WebSocket(WS_URL);

    ws.onopen = () => {
        setStatus(true);
        logEvent('WebSocket connected');
    };

    ws.onmessage = (ev) => {
        let msg = ev.data;
        try {
            const parsed = JSON.parse(msg);
            msg = JSON.stringify(parsed, null, 2);
        } catch {}
        logEvent(`WS msg: ${msg.substring(0, 200)}`);
    };

    ws.onclose = () => {
        setStatus(false);
        ws = null;
        logEvent('WebSocket disconnected');
    };

    ws.onerror = (e) => {
        logEvent('WebSocket error');
    };
}

function disconnectWs() {
    if (ws) {
        ws.close();
        ws = null;
    }
}

$('#btn-connect').addEventListener('click', connectWs);
$('#btn-disconnect').addEventListener('click', disconnectWs);
$('#btn-refresh').addEventListener('click', refreshSkills);

// Auto-refresh skills on load
refreshSkills();
