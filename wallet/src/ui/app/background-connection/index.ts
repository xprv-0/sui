import { PortStream } from './PortStream';
import {
    MSG_SENDER_EXTENSION,
    UI_TO_BACKGROUND_CHANNEL_NAME,
} from '_shared/messaging/constants';

import type { PermissionResponse } from './messages/payloads/PermissionResponse';
import type { SuiAddress } from '@mysten/sui.js';
import type { AppDispatch } from '_store';

export class BackgroundConnection {
    // public readonly initReady: Promise<boolean>;
    private _portStream: PortStream;
    private _dispatch: AppDispatch | null = null;

    constructor() {
        this._portStream = new PortStream(
            UI_TO_BACKGROUND_CHANNEL_NAME,
            MSG_SENDER_EXTENSION,
            false
        );
    }

    public permissionResponse(
        id: string,
        accounts: SuiAddress[],
        allowed: boolean,
        responseDate: string
    ) {
        this._portStream.sendMessage<PermissionResponse>({
            id,
            type: 'permission-response',
            accounts,
            allowed,
            responseDate,
        });
    }
}
