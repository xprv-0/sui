import {
    createAsyncThunk,
    createEntityAdapter,
    createSlice,
} from '@reduxjs/toolkit';

import type { SuiAddress } from '@mysten/sui.js';
import type { PayloadAction } from '@reduxjs/toolkit';
import type { PermissionRequest } from '_app/background-connection/messages/payloads/PermissionRequest';
import type { RootState } from '_redux/RootReducer';
import type { AppThunkConfig } from '_store/thunk-extras';

const permissionsAdapter = createEntityAdapter<PermissionRequest>({
    sortComparer: (a, b) => {
        const aDate = new Date(a.createdDate);
        const bDate = new Date(b.createdDate);
        return aDate.getTime() - bDate.getTime();
    },
});

const respondToPermissionRequest = createAsyncThunk<
    {
        id: string;
        accounts: SuiAddress[];
        allowed: boolean;
        responseDate: string;
    },
    { id: string; accounts: SuiAddress[]; allowed: boolean },
    AppThunkConfig
>(
    'respond-to-permission-request',
    ({ id, accounts, allowed }, { extra: { backgroundConnection } }) => {
        const responseDate = new Date().toISOString();
        backgroundConnection.permissionResponse(
            id,
            accounts,
            allowed,
            responseDate
        );
        return { id, accounts, allowed, responseDate };
    }
);

const slice = createSlice({
    name: 'permissions',
    initialState: permissionsAdapter.getInitialState({ initialized: false }),
    reducers: {
        setPermissions: (
            state,
            { payload }: PayloadAction<PermissionRequest[]>
        ) => {
            permissionsAdapter.setAll(state, payload);
            state.initialized = true;
        },
    },
    extraReducers: (build) => {
        build.addCase(
            respondToPermissionRequest.fulfilled,
            (state, { payload }) => {
                const { id, accounts, allowed, responseDate } = payload;
                permissionsAdapter.updateOne(state, {
                    id,
                    changes: {
                        accounts,
                        allowed,
                        responseDate,
                    },
                });
            }
        );
    },
});

export default slice.reducer;

export const { setPermissions } = slice.actions;

export const permissionsSelectors = permissionsAdapter.getSelectors(
    (state: RootState) => state.permissions
);
