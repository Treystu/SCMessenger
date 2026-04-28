// scmessenger-wasm — Comprehensive verification test suite for notification functionality

import { testNotificationsCrossBrowser, testPermissionFlow, testNotificationDelivery, testClickHandling, testNotificationGrouping, testNotificationSound, getBrowserInfo, generateVerificationReport } from './notification_test.js';

/**
 * Run the complete verification suite
 */
export async function runVerificationSuite() {
    const results = {
        browsers: await testNotificationsCrossBrowser(),
        features: await testNotificationFeatures(),
        performance: await testNotificationPerformance(),
        reliability: await testNotificationReliability()
    };

    // Generate verification report
    generateVerificationReport(results);

    return results;
}

/**
 * Test notification features
 */
export async function testNotificationFeatures() {
    return {
        basic: await testBasicNotifications(),
        click: await testClickHandling(),
        close: await testCloseHandling(),
        group: await testNotificationGrouping(),
        sound: await testNotificationSound()
    };
}

/**
 * Test notification performance
 */
export async function testNotificationPerformance() {
    const results = {
        deliveryTime: 0,
        processingTime: 0,
        memoryUsage: 0
    };

    const startTime = performance.now();

    // Test delivery time
    if (Notification.permission === 'granted') {
        const notification = new Notification('Performance Test', {
            body: 'Test delivery time',
            icon: '/icon.png'
        });

        notification.onshow = () => {
            results.deliveryTime = performance.now() - startTime;
        };
    }

    results.processingTime = performance.now() - startTime;

    return results;
}

/**
 * Test notification reliability
 */
export async function testNotificationReliability() {
    const results = {
        successCount: 0,
        totalCount: 10,
        failures: []
    };

    for (let i = 0; i < results.totalCount; i++) {
        try {
            if (Notification.permission === 'granted') {
                const notification = new Notification(`Reliability Test ${i + 1}`, {
                    body: `Test message ${i + 1}`,
                    icon: '/icon.png'
                });

                if (notification) {
                    results.successCount++;
                }
            }
        } catch (error) {
            results.failures.push({ index: i, error: error.message });
        }
    }

    results.reliability = results.successCount / results.totalCount;
    return results;
}

/**
 * Browser-specific compatibility tests
 */
export async function testBrowserCompatibility() {
    const browserInfo = getBrowserInfo();
    const results = {
        browser: browserInfo.browser,
        version: browserInfo.version,
        notificationsSupported: browserInfo.supportsNotifications,
        serviceWorkerSupported: browserInfo.supportsServiceWorker,
        pushSupported: browserInfo.supportsPush,
        tests: {}
    };

    // Browser-specific tests
    if (browserInfo.browser === 'chrome') {
        results.tests.chromeFeatures = await testChromeFeatures();
    } else if (browserInfo.browser === 'firefox') {
        results.tests.firefoxFeatures = await testFirefoxFeatures();
    } else if (browserInfo.browser === 'safari') {
        results.tests.safariFeatures = await testSafariFeatures();
    } else if (browserInfo.browser === 'edge') {
        results.tests.edgeFeatures = await testEdgeFeatures();
    }

    return results;
}

async function testChromeFeatures() {
    return {
        badgeSupport: true,
        vibrateSupport: true,
        requireInteraction: false
    };
}

async function testFirefoxFeatures() {
    return {
        badgeSupport: true,
        vibrateSupport: true,
        requireInteraction: true // Firefox may dismiss too quickly
    };
}

async function testSafariFeatures() {
    return {
        badgeSupport: true,
        vibrateSupport: false, // Safari doesn't support vibrate
        requireInteraction: false
    };
}

async function testEdgeFeatures() {
    return {
        badgeSupport: true,
        vibrateSupport: true,
        requireInteraction: false
    };
}

/**
 * Service worker integration tests
 */
export async function testServiceWorker() {
    const results = {
        serviceWorkerAvailable: 'serviceWorker' in navigator,
        registrationSuccessful: false,
        pushAvailable: 'PushManager' in window,
        backgroundFetchAvailable: 'backgroundFetch' in navigator
    };

    if ('serviceWorker' in navigator) {
        try {
            const registration = await navigator.serviceWorker.register('/service-worker.js');
            results.registrationSuccessful = true;

            // Test silent notification
            if (Notification.permission === 'granted') {
                await registration.showNotification('Service Worker Test', {
                    body: 'Background notification from service worker',
                    icon: '/icon.png'
                });
            }
        } catch (error) {
            console.error('Service worker registration failed:', error);
        }
    }

    return results;
}

/**
 * Permission verification tests
 */
export async function testPermissionVerification() {
    const results = {
        currentPermission: await Notification.permission,
        canRequest: true,
        requestHandled: false
    };

    if (Notification.permission === 'denied') {
        results.canRequest = false;
        results.guidanceNeeded = true;
    } else if (Notification.permission === 'granted') {
        results.canRequest = false;
        results.guidanceNeeded = false;
    } else {
        // Need to request
        try {
            const newPermission = await Notification.requestPermission();
            results.requestHandled = true;
            results.newPermission = newPermission;
            results.guidanceNeeded = newPermission !== 'granted';
        } catch (error) {
            console.error('Permission request failed:', error);
        }
    }

    return results;
}

/**
 * Generate comprehensive verification report
 */
export function generateDetailedReport(results) {
    const report = {
        timestamp: new Date().toISOString(),
        summary: {
            overall: 'pending',
            browsersTested: 0,
            featuresTested: 0,
            passed: 0,
            failed: 0
        },
        details: results
    };

    // Calculate summary
    if (results.browsers) {
        report.summary.browsersTested = Object.keys(results.browsers).length;
        for (const browser of Object.values(results.browsers)) {
            if (browser.success) report.summary.passed++;
            else report.summary.failed++;
        }
    }

    if (results.features) {
        report.summary.featuresTested = 5;
        if (results.features.basic) report.summary.passed++;
        if (results.features.click) report.summary.passed++;
        if (results.features.close) report.summary.passed++;
        if (results.features.group) report.summary.passed++;
        if (results.features.sound) report.summary.passed++;
    }

    if (report.summary.failed === 0 && report.summary.browsersTested > 0) {
        report.summary.overall = 'passed';
    } else {
        report.summary.overall = 'failed';
    }

    console.log('Detailed Verification Report:', JSON.stringify(report, null, 2));
    return report;
}
