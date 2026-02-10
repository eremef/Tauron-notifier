document.addEventListener('DOMContentLoaded', () => {
    initSettings();
    initPullToRefresh();
    loadSettingsAndFetch();
});

// ── Settings ──────────────────────────────────────────────

function initSettings() {
    const btn = document.getElementById('settings-btn');
    const panel = document.getElementById('settings-panel');
    const saveBtn = document.getElementById('save-settings-btn');

    btn.addEventListener('click', () => {
        panel.classList.toggle('hidden');
    });

    saveBtn.addEventListener('click', saveSettings);
}

async function loadSettingsAndFetch() {
    try {
        const settings = await window.__TAURI__.core.invoke('load_settings');
        if (settings) {
            document.getElementById('city-input').value = settings.cityName;
            document.getElementById('street-input').value = settings.streetName;
            document.getElementById('house-input').value = settings.houseNo;
            fetchOutages();
        } else {
            // No settings yet — show setup prompt
            const container = document.getElementById('outages-container');
            container.innerHTML = '<div class="no-outages">Tap ⚙️ to configure your location.</div>';
            document.getElementById('last-updated').textContent = 'Not configured';
            document.getElementById('settings-panel').classList.remove('hidden');
        }
    } catch (error) {
        console.error('Error loading settings:', error);
    }
}

async function saveSettings() {
    const cityName = document.getElementById('city-input').value.trim();
    const streetName = document.getElementById('street-input').value.trim();
    const houseNo = document.getElementById('house-input').value.trim();
    const status = document.getElementById('settings-status');

    if (!cityName || !streetName || !houseNo) {
        status.textContent = '⚠️ All fields are required.';
        status.className = 'settings-status error';
        return;
    }

    const saveBtn = document.getElementById('save-settings-btn');
    saveBtn.disabled = true;

    try {
        // Step 1: Lookup city
        status.textContent = '🔍 Looking up city...';
        status.className = 'settings-status';
        const cities = await window.__TAURI__.core.invoke('lookup_city', { cityName });

        const city = cities.find(c => c.Name === cityName);
        if (!city) {
            const available = cities.map(c => c.Name).join(', ');
            status.textContent = `❌ City not found. Did you mean: ${available || 'none'}?`;
            status.className = 'settings-status error';
            saveBtn.disabled = false;
            return;
        }

        // Step 2: Lookup street
        status.textContent = '🔍 Looking up street...';
        const streets = await window.__TAURI__.core.invoke('lookup_street', {
            streetName,
            cityGaid: city.GAID
        });

        const street = streets.find(s => s.Name === streetName);
        if (!street) {
            const available = streets.map(s => s.Name).join(', ');
            status.textContent = `❌ Street not found. Did you mean: ${available || 'none'}?`;
            status.className = 'settings-status error';
            saveBtn.disabled = false;
            return;
        }

        // Step 3: Save settings
        status.textContent = '💾 Saving...';
        await window.__TAURI__.core.invoke('save_settings', {
            settings: {
                cityName,
                streetName,
                houseNo,
                cityGAID: city.GAID,
                streetGAID: street.GAID
            }
        });

        status.textContent = `✅ Saved! City=${city.GAID}, Street=${street.GAID}`;
        status.className = 'settings-status success';

        // Collapse settings and refresh outages
        setTimeout(() => {
            document.getElementById('settings-panel').classList.add('hidden');
            status.textContent = '';
        }, 1500);

        fetchOutages();
    } catch (error) {
        status.textContent = `❌ ${error}`;
        status.className = 'settings-status error';
    } finally {
        saveBtn.disabled = false;
    }
}

// ── Pull to Refresh ───────────────────────────────────────

function initPullToRefresh() {
    const indicator = document.getElementById('pull-indicator');
    let startY = 0;
    let pulling = false;
    const threshold = 80;

    document.addEventListener('touchstart', (e) => {
        if (window.scrollY === 0) {
            startY = e.touches[0].clientY;
            pulling = true;
        }
    }, { passive: true });

    document.addEventListener('touchmove', (e) => {
        if (!pulling) return;
        const dy = e.touches[0].clientY - startY;
        if (dy > 10 && window.scrollY === 0) {
            indicator.classList.toggle('visible', dy > threshold / 2);
        }
    }, { passive: true });

    document.addEventListener('touchend', () => {
        if (!pulling) return;
        pulling = false;
        if (indicator.classList.contains('visible')) {
            indicator.classList.remove('visible');
            indicator.classList.add('refreshing');
            indicator.textContent = '↻ Refreshing...';
            fetchOutages().finally(() => {
                indicator.classList.remove('refreshing');
                indicator.textContent = '↻ Release to refresh';
            });
        }
    });
}

// ── Outages ───────────────────────────────────────────────

async function fetchOutages() {
    const container = document.getElementById('outages-container');
    const lastUpdated = document.getElementById('last-updated');

    try {
        const data = await window.__TAURI__.core.invoke('fetch_outages');
        lastUpdated.textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
        renderOutages(data, container);
    } catch (error) {
        console.error('Error fetching data:', error);
        container.innerHTML = `<div class="error">Failed to load outage data. Error: ${error}</div>`;
    }
}

function renderOutages(data, container) {
    const allOutages = data.OutageItems || [];

    let streetName = '';
    const streetInput = document.getElementById('street-input');
    if (streetInput && streetInput.value.trim()) {
        streetName = streetInput.value.trim();
    }

    const localOutages = allOutages.filter(item =>
        item.Message && item.Message.includes(streetName)
    );

    container.innerHTML = '';

    // Local outages section
    if (localOutages.length > 0) {
        container.innerHTML += `<div class="section-label">Your location (${localOutages.length})</div>`;
        container.innerHTML += renderCards(localOutages);
    } else {
        container.innerHTML += `<div class="no-outages">No planned outages for your location.</div>`;
    }

    // All outages section
    const otherOutages = allOutages.filter(item =>
        !item.Message || !item.Message.includes(streetName)
    );
    if (otherOutages.length > 0) {
        container.innerHTML += `<div class="section-label other">Other outages (${otherOutages.length})</div>`;
        container.innerHTML += renderCards(otherOutages);
    }
}

function renderCards(outages) {
    return outages.map(item => `
        <div class="card">
            <span class="outage-type">Planned Outage</span>
            <div class="outage-time">
                ${formatDate(item.StartDate)} – ${formatDate(item.EndDate)}
            </div>
            ${item.Description ? `<div class="outage-reason">${item.Description}</div>` : ''}
            ${item.Message ? `<div class="outage-message">${item.Message}</div>` : ''}
        </div>
    `).join('');
}

function formatDate(dateString) {
    if (!dateString) return '';
    const date = new Date(dateString);
    return date.toLocaleString('pl-PL', {
        weekday: 'short',
        day: 'numeric',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit'
    });
}
