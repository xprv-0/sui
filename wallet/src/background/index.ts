// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

import Browser from 'webextension-polyfill';

import { openInNewTab } from '_shared/utils';

import type { Runtime } from 'webextension-polyfill';

Browser.runtime.onInstalled.addListener((details) => {
    if (details.reason === 'install') {
        openInNewTab();
    }
});

const connectedPorts: Runtime.Port[] = [];

async function openWindow<T extends { id: string }>(origin: string, data: T) {
    const {
        width = 0,
        left = 0,
        top = 0,
    } = await Browser.windows.getLastFocused();
    return Browser.windows.create({
        url:
            Browser.runtime.getURL('ui.html') +
            `#/connect/${encodeURIComponent(data.id)}`,
        focused: true,
        width: 450,
        height: 600,
        type: 'popup',
        top: top,
        left: Math.floor(left + width - 450),
    });
}

Browser.runtime.onConnect.addListener((port, ...params) => {
    console.log('BG onConnect', { port, params });
    const origin = port.sender?.origin;
    if (origin) {
        console.log('bg script new connection', origin);
        port.onMessage.addListener(async (msg) => {
            console.log('Message from content script', msg);
            const newWindow = await openWindow(origin, msg);
            console.log('new window opened', newWindow);
            setTimeout(
                () => port.postMessage({ address: '0xtest', id: msg.id }),
                3000
            );
        });
        port.onDisconnect.addListener(() => {
            const index = connectedPorts.indexOf(port);
            console.log('BG script Port disconnected', { port, index });
            if (index >= 0) {
                connectedPorts.splice(index, 1);
                console.log('BG script Port removed', { connectedPorts });
            }
        });
        connectedPorts.push(port);
    } else {
        console.log('Origin not found. Disconnecting port', port);
        port.disconnect();
    }
});
