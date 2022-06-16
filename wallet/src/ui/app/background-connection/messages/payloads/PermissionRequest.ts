import type { BasePayload } from './BasePayload';
import type { SuiAddress } from '@mysten/sui.js';
import type { PermissionType } from '_app/background-connection/PermissionType';

export interface PermissionRequest extends BasePayload {
    type: 'permission-request';
    id: string;
    origin: string;
    accounts: SuiAddress[];
    allowed: boolean | null;
    permissions: PermissionType[];
    createdDate: string;
    responseDate: string;
}
