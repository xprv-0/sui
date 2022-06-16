import { nanoid } from '@reduxjs/toolkit';
import { Subject } from 'rxjs';
import Browser from 'webextension-polyfill';

import type { Message } from './messages/Message';
import type { BasePayload } from './messages/payloads/BasePayload';
import type { ErrorPayload } from './messages/payloads/ErrorPayload';
import type { Observable } from 'rxjs';
import type { Runtime } from 'webextension-polyfill';

export class PortStream {
    private _messagesSubject: Subject<Message>;
    private _messagesStream: Observable<Message>;
    private _channelName: string;
    private _connected: boolean;
    private _bgPort: Runtime.Port | null;
    private _sender: string;

    constructor(channelName: string, sender: string, lazyConnect = true) {
        this._messagesSubject = new Subject();
        this._messagesStream = this._messagesSubject.asObservable();
        this._connected = false;
        this._bgPort = null;
        this._channelName = channelName;
        this._sender = sender;
        if (!lazyConnect) {
            this.connect();
        }
    }

    public get onMessage(): Observable<Message> {
        if (!this._connected) {
            this.connect();
        }
        return this._messagesStream;
    }

    public sendMessage<T extends BasePayload, E = void>(
        msgPayload: T | ErrorPayload<E>,
        responseForID?: string
    ) {
        if (!this._connected) {
            this.connect();
        }
        if (!this._bgPort) {
            console.error('[PortStream] port expected to be defined');
            throw new Error('Port to background service worker is not defined');
        }
        const msg: Message<T, E> = {
            id: nanoid(),
            responseForID,
            sender: this._sender,
            payload: msgPayload,
        };
        this._bgPort.postMessage(msgPayload);
    }

    private connect() {
        this._connected = true;
        this._bgPort = Browser.runtime.connect({
            name: this._channelName,
        });
        this._bgPort.onMessage.addListener((msg) => {
            console.log('[PortStream] Received message:', msg);
            this._messagesSubject.next(msg);
        });
        this._bgPort.onDisconnect.addListener(() => {
            console.log('[PortStream] Port disconnected');
            this._connected = false;
            this._bgPort = null;
            this.connect(); // TODO: maybe auto connect flag or retry with back-off strategy
        });
    }
}
