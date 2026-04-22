// scmessenger-wasm — Browser-specific notification workarounds and compatibility

/**
 * Get browser-specific notification options
 */
export function getBrowserSpecificNotificationOptions() {
    const userAgent = navigator.userAgent.toLowerCase();

    if (userAgent.includes('edg') || userAgent.includes('edge')) {
        return {
            requireInteraction: false,
            badge: '/badge.png',
            vibrate: [200, 100, 200],
            tag: 'scmessenger_edge'
        };
    } else if (userAgent.includes('safari') && !userAgent.includes('chrome')) {
        return {
            // Safari has stricter requirements
            tag: 'scmessenger_safari',
            renotify: true,
            badge: '/badge.png'
        };
    } else if (userAgent.includes('firefox')) {
        return {
            requireInteraction: true, // Firefox may dismiss too quickly
            tag: 'scmessenger_firefox',
            badge: '/badge.png',
            vibrate: [200, 100, 200]
        };
    } else if (userAgent.includes('chrome') || userAgent.includes('chromium')) {
        return {
            requireInteraction: false,
            badge: '/badge.png',
            vibrate: [200, 100, 200],
            icon: '/icon.png'
        };
    } else {
        return {
            // Default options for unknown browsers
            tag: 'scmessenger',
            badge: '/badge.png'
        };
    }
}

/**
 * Get browser type from user agent
 */
export function getBrowserType() {
    const userAgent = navigator.userAgent.toLowerCase();

    if (userAgent.includes('edg') || userAgent.includes('edge')) {
        return { name: 'edge', version: extractVersion(userAgent, 'edg') };
    } else if (userAgent.includes('safari') && !userAgent.includes('chrome')) {
        return { name: 'safari', version: extractVersion(userAgent, 'version') };
    } else if (userAgent.includes('firefox')) {
        return { name: 'firefox', version: extractVersion(userAgent, 'firefox') };
    } else if (userAgent.includes('chrome') || userAgent.includes('chromium')) {
        return { name: 'chrome', version: extractVersion(userAgent, 'chrome') };
    } else {
        return { name: 'unknown', version: 'unknown' };
    }
}

/**
 * Extract browser version from user agent
 */
function extractVersion(userAgent, marker) {
    const regex = new RegExp(marker + '/([0-9]+)');
    const match = userAgent.match(regex);
    return match ? match[1] : 'unknown';
}

/**
 * Check if browser requires special handling
 */
export function requiresSpecialHandling() {
    const browser = getBrowserType();

    // Safari requires user interaction for some features
    if (browser.name === 'safari') {
        return true;
    }

    // Firefox may dismiss notifications too quickly
    if (browser.name === 'firefox') {
        return true;
    }

    return false;
}

/**
 * Get notification guidance for denied permissions
 */
export function getPermissionGuidanceHTML() {
    const browser = getBrowserType();
    let instructions = '';

    switch (browser.name) {
        case 'chrome':
        case 'edge':
            instructions = `Open Chrome Settings > Privacy and Security > Site Settings > Notifications`;
            break;
        case 'firefox':
            instructions = `Open Firefox Preferences > Privacy & Security > Permissions > Notifications`;
            break;
        case 'safari':
            instructions = `Open Safari Preferences > Websites > Notifications`;
            break;
        default:
            instructions = `Open your browser settings and allow notifications for this site`;
    }

    return `
        <div style="
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            z-index: 9999;
            max-width: 450px;
        ">
            <h3 style="margin-top: 0; color: #333;">Notifications Disabled</h3>
            <p style="margin-bottom: 1rem; color: #666;">
                Please enable notifications in your browser settings to receive messages from SCMessenger.
            </p>
            <div style="
                background: #f8f9fa;
                padding: 1rem;
                border-radius: 4px;
                margin: 1rem 0;
                font-size: 0.9rem;
                color: #495057;
                border-left: 4px solid #007bff;
            ">
                <strong>Instructions:</strong><br>
                ${instructions}
            </div>
            <div style="display: flex; gap: 10px;">
                <button id="open-settings-btn" style="
                    flex: 1;
                    padding: 10px 20px;
                    background: #007bff;
                    color: white;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 0.9rem;
                ">Open Settings</button>
                <button id="close-guidance-btn" style="
                    flex: 1;
                    padding: 10px 20px;
                    background: #6c757d;
                    color: white;
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                    font-size: 0.9rem;
                ">Cancel</button>
            </div>
        </div>
    `;
}

/**
 * Browser compatibility map for notification features
 */
export const BrowserFeatureMap = {
    chrome: {
        badge: true,
        vibrate: true,
        icon: true,
        tag: true,
        renotify: true,
        requireInteraction: true
    },
    firefox: {
        badge: true,
        vibrate: true,
        icon: true,
        tag: true,
        renotify: true,
        requireInteraction: true
    },
    safari: {
        badge: true,
        vibrate: false,
        icon: true,
        tag: true,
        renotify: true,
        requireInteraction: false
    },
    edge: {
        badge: true,
        vibrate: true,
        icon: true,
        tag: true,
        renotify: true,
        requireInteraction: true
    }
};

/**
 * Check if current browser supports a specific feature
 */
export function isFeatureSupported(feature) {
    const browser = getBrowserType();
    const features = BrowserFeatureMap[browser.name];

    if (!features) {
        return false;
    }

    return features[feature] !== undefined ? features[feature] : false;
}

/**
 * Get optimal notification settings for current browser
 */
export function getOptimalNotificationSettings() {
    const browser = getBrowserType();
    const features = BrowserFeatureMap[browser.name] || BrowserFeatureMap['chrome'];

    const settings = {
        icon: '/icon.png',
        badge: '/badge.png',
        tag: `scmessenger_${browser.name}`,
        requireInteraction: features.requireInteraction ? true : undefined
    };

    if (features.vibrate) {
        settings.vibrate = [200, 100, 200];
    }

    if (features.renotify) {
        settings.renotify = true;
    }

    return settings;
}

/**
 * Show permission guidance UI
 */
export function showPermissionGuidance() {
    const guidanceHTML = getPermissionGuidanceHTML();

    // Remove existing guidance if present
    const existing = document.getElementById('scmessenger-notification-guidance');
    if (existing) {
        existing.remove();
    }

    // Create guidance container
    const container = document.createElement('div');
    container.id = 'scmessenger-notification-guidance';
    container.innerHTML = guidanceHTML;

    document.body.appendChild(container);

    // Setup event handlers
    const openBtn = document.getElementById('open-settings-btn');
    const closeBtn = document.getElementById('close-guidance-btn');

    if (openBtn) {
        openBtn.addEventListener('click', () => {
            openSettings();
        });
    }

    if (closeBtn) {
        closeBtn.addEventListener('click', () => {
            document.body.removeChild(container);
        });
    }

    return container;
}

/**
 * Open browser settings for notifications
 */
export function openSettings() {
    const browser = getBrowserType();
    let url = '';

    switch (browser.name) {
        case 'chrome':
        case 'edge':
            url = 'chrome://settings/content/notifications';
            break;
        case 'firefox':
            url = 'about:preferences#permissions';
            break;
        case 'safari':
            url = 'safari://preferences/notifications';
            break;
        default:
            // For other browsers, try to open generic settings
            url = window.location.origin + '/settings';
    }

    // Note: Browser settings URLs may not work from all contexts
    // The user may need to navigate manually
    window.open(url, '_blank');
}
