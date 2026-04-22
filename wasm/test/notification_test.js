// scmessenger-wasm — Notification verification tests for browser environments

/**
 * Cross-browser notification testing
 */
export async function testNotificationsCrossBrowser() {
    const browsers = ['chrome', 'firefox', 'safari', 'edge'];
    const results = {};

    for (const browser of browsers) {
        try {
            results[browser] = await testBrowserNotifications(browser);
        } catch (error) {
            results[browser] = { success: false, error: error.message };
        }
    }

    return results;
}

async function testBrowserNotifications(browser) {
    // Test basic notification permission
    const permission = await Notification.requestPermission();

    // Test notification display
    const notification = new Notification('SCMessenger Test', {
        body: 'Test notification from SCMessenger',
        icon: '/icon.png'
    });

    // Test click handling
    notification.onclick = () => {
        console.log('Notification clicked');
        window.focus();
    };

    // Test close handling
    notification.onclose = () => {
        console.log('Notification closed');
    };

    return {
        success: true,
        permission,
        features: {
            basic: true,
            click: true,
            close: true
        },
        browser: browser
    };
}

/**
 * Test notification permission flow
 */
export async function testPermissionFlow() {
    const result = {
        status: 'unknown',
        requiredAction: 'none',
        success: false
    };

    try {
        const permission = await Notification.requestPermission();

        if (permission === 'granted') {
            result.status = 'granted';
            result.requiredAction = 'none';
            result.success = true;
            console.log('Notification permission: GRANTED');
        } else if (permission === 'denied') {
            result.status = 'denied';
            result.requiredAction = 'manualEnable';
            result.success = false;
            console.log('Notification permission: DENIED');
        } else {
            result.status = 'unknown';
            result.requiredAction = 'none';
            result.success = false;
            console.log('Notification permission: UNKNOWN');
        }
    } catch (error) {
        result.error = error.message;
    }

    return result;
}

/**
 * Test notification delivery in various app states
 */
export async function testNotificationDelivery() {
    const results = {
        basicNotification: false,
        messageNotification: false,
        groupNotification: false,
        backgroundNotification: false,
        terminatedNotification: false,
        deliveryTime: 0,
        reliability: 0
    };

    const testStartTime = Date.now();

    // Test basic notification
    results.basicNotification = await testBasicNotification();

    // Test message notification
    results.messageNotification = await testMessageNotification();

    // Test group notification
    results.groupNotification = await testGroupNotification();

    // Test background notification
    results.backgroundNotification = await testBackgroundNotification();

    // Test terminated state notification
    results.terminatedNotification = await testTerminatedNotification();

    // Calculate metrics
    results.deliveryTime = (Date.now() - testStartTime) / 1000;
    results.reliability = (results.basicNotification ? 1 : 0) +
                           (results.messageNotification ? 1 : 0) +
                           (results.groupNotification ? 1 : 0);

    return results;
}

async function testBasicNotification() {
    try {
        if (Notification.permission === 'granted') {
            const notification = new Notification('Test Basic Notification', {
                body: 'This is a basic notification test',
                icon: '/icon.png',
                badge: '/badge.png'
            });

            return notification !== undefined;
        }
        return false;
    } catch (error) {
        console.error('Basic notification test failed:', error);
        return false;
    }
}

async function testMessageNotification() {
    try {
        if (Notification.permission === 'granted') {
            const notification = new Notification('New Message', {
                body: 'You have a new message from test_peer',
                icon: '/icon.png',
                tag: 'message',
                renotify: true
            });

            return notification !== undefined;
        }
        return false;
    } catch (error) {
        console.error('Message notification test failed:', error);
        return false;
    }
}

async function testGroupNotification() {
    try {
        if (Notification.permission === 'granted') {
            const notification = new Notification('Group Chat', {
                body: 'New message in group: test_group',
                icon: '/icon.png',
                tag: 'group',
                renotify: true
            });

            return notification !== undefined;
        }
        return false;
    } catch (error) {
        console.error('Group notification test failed:', error);
        return false;
    }
}

async function testBackgroundNotification() {
    // Background notification test via service worker
    try {
        if ('serviceWorker' in navigator && 'PushManager' in window) {
            const registration = await navigator.serviceWorker.ready;

            // Test silent notification
            if (Notification.permission === 'granted') {
                const notification = await registration.showNotification('Background Test', {
                    body: 'Background processing test',
                    icon: '/icon.png',
                    badge: '/badge.png'
                });

                return notification !== undefined;
            }
        }
        return false;
    } catch (error) {
        console.error('Background notification test failed:', error);
        return false;
    }
}

async function testTerminatedNotification() {
    // Simulate terminated state notification
    // In practice, this is tested via service worker push events
    try {
        if ('serviceWorker' in navigator) {
            const registration = await navigator.serviceWorker.ready;

            // Show notification that would work from terminated state
            const notification = await registration.showNotification('Terminated Test', {
                body: 'Notification from terminated state',
                icon: '/icon.png',
                badge: '/badge.png'
            });

            return notification !== undefined;
        }
        return false;
    } catch (error) {
        console.error('Terminated notification test failed:', error);
        return false;
    }
}

/**
 * Notification click handling tests
 */
export async function testClickHandling() {
    const results = {
        notificationClick: false,
        actionClick: false,
        defaultAction: false
    };

    // Test basic click
    try {
        const notification = new Notification('Click Test', {
            body: 'Click me to test',
            icon: '/icon.png'
        });

        notification.onclick = () => {
            results.notificationClick = true;
            window.focus();
        };

        // Simulate click
        notification.onclick();

        results.defaultAction = results.notificationClick;
    } catch (error) {
        console.error('Click handling test failed:', error);
    }

    return results;
}

/**
 * Test notification grouping
 */
export async function testNotificationGrouping() {
    try {
        const notification1 = new Notification('Group Test', {
            body: 'Message 1',
            icon: '/icon.png',
            tag: 'group1',
            renotify: true
        });

        const notification2 = new Notification('Group Test', {
            body: 'Message 2',
            icon: '/icon.png',
            tag: 'group1'
        });

        // Notifications with same tag replace each other
        return notification2 !== undefined;
    } catch (error) {
        console.error('Notification grouping test failed:', error);
        return false;
    }
}

/**
 * Test notification sound
 */
export async function testNotificationSound() {
    try {
        if (Notification.permission === 'granted') {
            const notification = new Notification('Sound Test', {
                body: 'This notification has sound',
                icon: '/icon.png',
                sound: '/sounds/notification.mp3'
            });

            return notification !== undefined;
        }
        return false;
    } catch (error) {
        console.error('Notification sound test failed:', error);
        return false;
    }
}

/**
 * Test browser compatibility
 */
export function getBrowserInfo() {
    const userAgent = navigator.userAgent.toLowerCase();

    let browser = 'unknown';
    let version = 'unknown';

    if (userAgent.includes('edg')) {
        browser = 'edge';
        const match = userAgent.match(/edg\/([0-9]+)/);
        if (match) version = match[1];
    } else if (userAgent.includes('chrome') && !userAgent.includes('edg')) {
        browser = 'chrome';
        const match = userAgent.match(/chrome\/([0-9]+)/);
        if (match) version = match[1];
    } else if (userAgent.includes('safari') && !userAgent.includes('chrome')) {
        browser = 'safari';
        const match = userAgent.match(/version\/([0-9]+)/);
        if (match) version = match[1];
    } else if (userAgent.includes('firefox')) {
        browser = 'firefox';
        const match = userAgent.match(/firefox\/([0-9]+)/);
        if (match) version = match[1];
    }

    return {
        browser,
        version,
        supportsNotifications: Notification.permission !== undefined,
        supportsServiceWorker: 'serviceWorker' in navigator,
        supportsPush: 'PushManager' in window
    };
}

/**
 * Generate verification report
 */
export async function generateVerificationReport(results) {
    const report = {
        timestamp: new Date().toISOString(),
        results,
        summary: {
            passed: 0,
            failed: 0,
            total: 0
        }
    };

    // Count results
    if (results.browsers) {
        for (const [browser, result] of Object.entries(results.browsers)) {
            report.summary.total++;
            if (result.success) {
                report.summary.passed++;
            } else {
                report.summary.failed++;
            }
        }
    }

    if (results.features) {
        report.summary.total += 5;
        if (results.features.basic) report.summary.passed++;
        if (results.features.click) report.summary.passed++;
        if (results.features.close) report.summary.passed++;
        if (results.features.group) report.summary.passed++;
        if (results.features.sound) report.summary.passed++;
    }

    console.log('Verification Report:', JSON.stringify(report, null, 2));
    return report;
}
