import type { BasePayload } from './BasePayload';
import type { SuiAddress } from '@mysten/sui.js';

export interface PermissionResponse extends BasePayload {
    type: 'permission-response';
    id: string;
    accounts: SuiAddress[];
    allowed: boolean;
    responseDate: string;
}
