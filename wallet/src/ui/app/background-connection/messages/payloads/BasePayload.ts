export type PayloadType = 'permission-request' | 'permission-response';

export interface BasePayload {
    type: PayloadType;
}
