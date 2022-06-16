import { nanoid } from '@reduxjs/toolkit';
import { filter, fromEvent, lastValueFrom, map, take, tap } from 'rxjs';

import {
    MSG_SENDER_CONTENT,
    MSG_SENDER_DAPP,
    MSG_SENDER_FIELD,
} from '_shared/messaging/constants';

import type { SuiAddress } from '@mysten/sui.js';
import type { Observable } from 'rxjs';

export class DAppInterface {
    private _window: Window;
    private _messagesStream: Observable<MessageEvent>;

    constructor(theWindow: Window) {
        this._window = window;
        this._messagesStream = fromEvent<MessageEvent>(
            theWindow,
            'message'
        ).pipe(
            tap((e) => console.log('Dapp Received event msg', e.data)),
            filter(
                (e) =>
                    e.source === theWindow &&
                    e.data[MSG_SENDER_FIELD] === MSG_SENDER_CONTENT
            )
        );
    }

    public getAccount(): Promise<SuiAddress> {
        const id = nanoid();
        const stream = this._messagesStream.pipe(
            filter((e) => e.data.id === id),
            tap((e) => console.log('Got response message for id', id, e.data)),
            map((e) => e.data.address as SuiAddress),
            take(1)
        );
        this._window.postMessage({
            type: 'getAccount',
            id,
            [MSG_SENDER_FIELD]: MSG_SENDER_DAPP,
        });
        return lastValueFrom(stream);
    }
}
