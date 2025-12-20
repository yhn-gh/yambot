/**
 * Yambot Overlay - Main Script
 * Handles WebSocket connection and event processing
 */

// Configuration
const WS_URL = 'ws://localhost:3000/ws';
const RECONNECT_INTERVAL = 3000; // 3 seconds
const DEBUG_MODE = false; // Set to true to show debug panel

// Global state
let ws = null;
let reconnectTimeout = null;
let wheel = null;
let configMode = false;

// Drag state
let dragElement = null;
let isDragging = false;
let dragStartX = 0;
let dragStartY = 0;
let elementStartX = 0;
let elementStartY = 0;

// Resize state
let resizeElement = null;
let isResizing = false;
let resizeStartScale = 1;
let resizeStartX = 0;
let resizeStartY = 0;

// Initialize on page load
document.addEventListener('DOMContentLoaded', () => {
    console.log('Yambot Overlay Initialized');

    // Initialize spinning wheel
    wheel = new SpinningWheel('wheel-canvas', 'wheel-container');

    // Show debug panel if enabled
    if (DEBUG_MODE) {
        document.getElementById('debug-info').classList.remove('hidden');
    }

    // Connect to WebSocket
    connect();

    // Listen for config mode toggle (Press 'C' key)
    document.addEventListener('keydown', (e) => {
        if (e.key === 'c' || e.key === 'C') {
            toggleConfigMode();
        }
    });

    // Check if config mode is requested via URL param
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.get('config') === 'true') {
        toggleConfigMode();
    }
});

/**
 * Connect to the WebSocket server
 */
function connect() {
    console.log('Connecting to WebSocket...');
    updateDebug('connection', 'Connecting...');

    try {
        ws = new WebSocket(WS_URL);

        ws.onopen = () => {
            console.log('WebSocket connected');
            updateDebug('connection', 'Connected');

            // Clear reconnect timeout
            if (reconnectTimeout) {
                clearTimeout(reconnectTimeout);
                reconnectTimeout = null;
            }

            // Request current configuration
            send({ type: 'request_config' });
        };

        ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                console.log('Received event:', data);
                updateDebug('event', data.type || 'unknown');
                handleEvent(data);
            } catch (error) {
                console.error('Failed to parse message:', error);
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            updateDebug('connection', 'Error');
        };

        ws.onclose = () => {
            console.log('WebSocket disconnected');
            updateDebug('connection', 'Disconnected');
            ws = null;

            // Attempt to reconnect
            reconnectTimeout = setTimeout(() => {
                console.log('Attempting to reconnect...');
                connect();
            }, RECONNECT_INTERVAL);
        };
    } catch (error) {
        console.error('Failed to connect:', error);
        updateDebug('connection', 'Failed');

        // Attempt to reconnect
        reconnectTimeout = setTimeout(() => {
            connect();
        }, RECONNECT_INTERVAL);
    }
}

/**
 * Handle incoming events from the server
 */
function handleEvent(event) {
    switch (event.type) {
        case 'trigger_action':
            handleTriggerAction(event);
            break;

        case 'config_update':
            handleConfigUpdate(event);
            break;

        case 'ping':
            // Just a keep-alive, no action needed
            break;

        default:
            console.warn('Unknown event type:', event.type);
    }
}

/**
 * Handle trigger action events
 */
function handleTriggerAction(event) {
    const { action_type, data } = event;

    switch (action_type) {
        case 'spin_wheel':
            if (data.items && Array.isArray(data.items)) {
                wheel.setItems(data.items);
                wheel.spinAndShow(data);
            }
            break;

        default:
            console.warn('Unknown action type:', action_type);
    }
}

/**
 * Handle configuration update from server
 */
function handleConfigUpdate(event) {
    const { positions } = event;
    if (positions) {
        applyPositions(positions);
    }
}

/**
 * Apply position configuration to overlay elements
 */
function applyPositions(positions) {
    if (positions.wheel) {
        const wheelContainer = document.getElementById('wheel-container');
        wheelContainer.style.left = `${positions.wheel.x}%`;
        wheelContainer.style.top = `${positions.wheel.y}%`;
        const scale = positions.wheel.scale || 1;
        wheelContainer.style.transform = `translate(-50%, -50%) scale(${scale})`;
        wheelContainer.dataset.scale = scale;
    }
}

/**
 * Toggle configuration mode
 */
function toggleConfigMode() {
    configMode = !configMode;
    document.body.classList.toggle('config-mode', configMode);

    if (configMode) {
        showConfigPanel();
        makeElementsDraggable();
    } else {
        hideConfigPanel();
        removeElementsDraggable();
    }
}

/**
 * Show configuration panel
 */
function showConfigPanel() {
    let panel = document.getElementById('config-panel');
    if (!panel) {
        panel = document.createElement('div');
        panel.id = 'config-panel';
        panel.className = 'config-panel';
        panel.innerHTML = `
            <h3>Overlay Configuration</h3>
            <p>Drag elements to reposition them.</p>
            <p>Use corner handles to resize.</p>
            <p>Press 'C' to exit config mode.</p>
            <button onclick="resetPositions()">Reset Positions</button>
        `;
        document.body.appendChild(panel);
    }
    panel.classList.remove('hidden');
}

/**
 * Hide configuration panel
 */
function hideConfigPanel() {
    const panel = document.getElementById('config-panel');
    if (panel) {
        panel.classList.add('hidden');
    }
}

/**
 * Make overlay elements draggable
 */
function makeElementsDraggable() {
    const elements = [
        { id: 'wheel-container', name: 'wheel' }
    ];

    elements.forEach(({ id, name }) => {
        const element = document.getElementById(id);
        if (element) {
            element.classList.add('draggable');
            element.setAttribute('data-element-name', name);

            // Add resize handles
            addResizeHandles(element);

            // Add scale indicator
            addScaleIndicator(element);

            element.addEventListener('mousedown', startDrag);
        }
    });
}

/**
 * Add resize handles to an element
 */
function addResizeHandles(element) {
    const positions = ['bottom-right'];

    positions.forEach(pos => {
        const handle = document.createElement('div');
        handle.className = `resize-handle ${pos}`;
        handle.dataset.position = pos;
        handle.addEventListener('mousedown', startResize);
        element.appendChild(handle);
    });
}

/**
 * Add scale indicator to an element
 */
function addScaleIndicator(element) {
    const indicator = document.createElement('div');
    indicator.className = 'scale-indicator';
    const scale = parseFloat(element.dataset.scale || 1);
    indicator.textContent = `Scale: ${(scale * 100).toFixed(0)}%`;
    element.appendChild(indicator);
}

/**
 * Remove draggable functionality
 */
function removeElementsDraggable() {
    const elements = document.querySelectorAll('.draggable');
    elements.forEach(element => {
        element.classList.remove('draggable');
        element.removeEventListener('mousedown', startDrag);

        // Remove resize handles
        const handles = element.querySelectorAll('.resize-handle');
        handles.forEach(handle => handle.remove());

        // Remove scale indicator
        const indicator = element.querySelector('.scale-indicator');
        if (indicator) indicator.remove();
    });
}

function startDrag(e) {
    // Don't start drag if clicking on resize handle
    if (e.target.classList.contains('resize-handle')) {
        return;
    }

    if (!configMode) return;

    dragElement = e.currentTarget;
    isDragging = true;

    // Get current position
    const rect = dragElement.getBoundingClientRect();

    // Store start positions (center of element in viewport coordinates)
    dragStartX = e.clientX;
    dragStartY = e.clientY;

    // Store element's current center position as percentage
    elementStartX = ((rect.left + rect.width / 2) / window.innerWidth) * 100;
    elementStartY = ((rect.top + rect.height / 2) / window.innerHeight) * 100;

    document.addEventListener('mousemove', drag);
    document.addEventListener('mouseup', stopDrag);
    e.preventDefault();
    e.stopPropagation();
}

function drag(e) {
    if (!dragElement || !isDragging) return;

    // Calculate mouse movement in pixels
    const deltaX = e.clientX - dragStartX;
    const deltaY = e.clientY - dragStartY;

    // Convert to percentage of viewport
    const deltaXPercent = (deltaX / window.innerWidth) * 100;
    const deltaYPercent = (deltaY / window.innerHeight) * 100;

    // Calculate new position
    const newX = elementStartX + deltaXPercent;
    const newY = elementStartY + deltaYPercent;

    // Apply position (centered)
    dragElement.style.left = `${newX}%`;
    dragElement.style.top = `${newY}%`;
    dragElement.style.right = 'auto';
    dragElement.style.bottom = 'auto';
}

function stopDrag(e) {
    if (!dragElement || !isDragging) return;

    const rect = dragElement.getBoundingClientRect();
    const x = ((rect.left + rect.width / 2) / window.innerWidth) * 100;
    const y = ((rect.top + rect.height / 2) / window.innerHeight) * 100;

    const elementName = dragElement.getAttribute('data-element-name');
    const scale = parseFloat(dragElement.dataset.scale || 1);

    // Send position update to backend (including scale)
    send({
        type: 'position_update',
        element: elementName,
        x: x,
        y: y,
        scale: scale
    });

    document.removeEventListener('mousemove', drag);
    document.removeEventListener('mouseup', stopDrag);
    dragElement = null;
    isDragging = false;
}

function startResize(e) {
    if (!configMode) return;

    e.stopPropagation();
    e.preventDefault();

    resizeElement = e.target.parentElement;
    isResizing = true;

    resizeStartX = e.clientX;
    resizeStartY = e.clientY;
    resizeStartScale = parseFloat(resizeElement.dataset.scale || 1);

    document.addEventListener('mousemove', resize);
    document.addEventListener('mouseup', stopResize);
}

function resize(e) {
    if (!resizeElement || !isResizing) return;

    // Calculate distance moved
    const deltaX = e.clientX - resizeStartX;
    const deltaY = e.clientY - resizeStartY;

    // Use the larger delta for uniform scaling
    const delta = Math.max(deltaX, deltaY);

    // Calculate scale change (every 100px = 0.5x scale change)
    const scaleChange = delta / 200;
    let newScale = resizeStartScale + scaleChange;

    // Clamp scale between 0.3 and 3.0
    newScale = Math.max(0.3, Math.min(3.0, newScale));

    // Apply scale
    resizeElement.dataset.scale = newScale;
    const currentTransform = resizeElement.style.transform || '';

    // Update transform while preserving translate
    if (currentTransform.includes('translate')) {
        // Extract translate part
        const translateMatch = currentTransform.match(/translate\([^)]+\)/);
        if (translateMatch) {
            resizeElement.style.transform = `${translateMatch[0]} scale(${newScale})`;
        }
    } else {
        resizeElement.style.transform = `scale(${newScale})`;
    }

    // Update scale indicator
    const indicator = resizeElement.querySelector('.scale-indicator');
    if (indicator) {
        indicator.textContent = `Scale: ${(newScale * 100).toFixed(0)}%`;
    }
}

function stopResize(e) {
    if (!resizeElement || !isResizing) return;

    const elementName = resizeElement.getAttribute('data-element-name');
    const newScale = parseFloat(resizeElement.dataset.scale || 1);

    // Get current position
    const rect = resizeElement.getBoundingClientRect();
    const x = ((rect.left + rect.width / 2) / window.innerWidth) * 100;
    const y = ((rect.top + rect.height / 2) / window.innerHeight) * 100;

    // Send position update to backend (including scale)
    send({
        type: 'position_update',
        element: elementName,
        x: x,
        y: y,
        scale: newScale
    });

    document.removeEventListener('mousemove', resize);
    document.removeEventListener('mouseup', stopResize);
    resizeElement = null;
    isResizing = false;
}

/**
 * Reset positions to defaults
 */
function resetPositions() {
    const defaults = {
        wheel: { x: 50, y: 50, scale: 1.0 }
    };

    Object.entries(defaults).forEach(([name, pos]) => {
        send({
            type: 'position_update',
            element: name,
            x: pos.x,
            y: pos.y,
            scale: pos.scale
        });
    });

    // Apply immediately
    applyPositions(defaults);
}

/**
 * Update debug info
 */
function updateDebug(field, value) {
    if (!DEBUG_MODE) return;

    const element = document.getElementById(`debug-${field}`);
    if (element) {
        element.textContent = value;
    }
}

/**
 * Send a message to the server
 */
function send(data) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(data));
    } else {
        console.warn('WebSocket not connected, cannot send message');
    }
}

/**
 * Export send function for wheel to use
 */
window.sendToBackend = send;

// Cleanup on page unload
window.addEventListener('beforeunload', () => {
    if (ws) {
        ws.close();
    }
    if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
    }
});
