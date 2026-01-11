import { apiClient } from './apiClient';
import type { ApiResponse } from '../types/api';
import type { Inbound } from '../types/inbound';


export type { Inbound };

export interface CreateInboundRequest {
    port: number;
    protocol: string;
    remark: string;
    settings?: any;
    streamSettings?: any;
    sniffing?: any;
}

export interface UpdateInboundRequest {
    port?: number;
    protocol?: string;
    remark?: string;
    enable?: boolean;
    settings?: any;
    streamSettings?: any;
    sniffing?: any;
}

export const inboundApi = {

    getInbounds: async (): Promise<ApiResponse<Inbound[]>> => {
        const response = await apiClient.get<ApiResponse<Inbound[]>>(`/inbound/list?_t=${Date.now()}`);
        return response.data;
    },


    getInbound: async (id: string): Promise<ApiResponse<Inbound>> => {
        const response = await apiClient.get<ApiResponse<Inbound>>(`/inbound/get?id=${id}`);
        return response.data;
    },


    createInbound: async (data: CreateInboundRequest): Promise<ApiResponse<Inbound>> => {
        const response = await apiClient.post<ApiResponse<Inbound>>('/inbound/add', data);
        return response.data;
    },


    updateInbound: async (id: string, data: UpdateInboundRequest): Promise<ApiResponse<Inbound>> => {
        const response = await apiClient.post<ApiResponse<Inbound>>('/inbound/update', { ...data, id });
        return response.data;
    },


    deleteInbound: async (id: string): Promise<ApiResponse<void>> => {
        const response = await apiClient.post<ApiResponse<void>>('/inbound/del', { id });
        return response.data;
    },


    toggleInbound: async (id: string, enable: boolean): Promise<ApiResponse<Inbound>> => {
        const response = await apiClient.post<ApiResponse<Inbound>>('/inbound/update', { id, enable });
        return response.data;
    },

    checkReality: async (domain: string): Promise<ApiResponse<{ is_valid: boolean, has_tls13: boolean, key_exchange: string, latency: number, message: string }>> => {
        const response = await apiClient.post<ApiResponse<{ is_valid: boolean, has_tls13: boolean, key_exchange: string, latency: number, message: string }>>('/inbound/check-reality', { domain });
        return response.data;
    },

    resetTraffic: async (id: string): Promise<ApiResponse<void>> => {
        const response = await apiClient.post<ApiResponse<void>>('/inbound/reset-traffic', { id });
        return response.data;
    },

    resetAllTraffic: async (): Promise<ApiResponse<void>> => {
        const response = await apiClient.post<ApiResponse<void>>('/inbound/reset-all');
        return response.data;
    },
};


