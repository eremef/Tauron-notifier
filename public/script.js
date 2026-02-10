document.addEventListener('DOMContentLoaded', () => {
    fetchOutages();
    initPullToRefresh();
});

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
    const localOutages = allOutages.filter(item =>
        item.Message && item.Message.includes("Rozbrat")
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
        !item.Message || !item.Message.includes("Rozbrat")
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
