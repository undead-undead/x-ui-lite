import { create } from 'zustand';
import type { Inbound } from '../types/inbound';
import { inboundApi } from '../api/inbound';
import { useDialogStore } from './useDialogStore';

interface InboundStore {
    inbounds: Inbound[];
    loading: boolean;
    fetchInbounds: (background?: boolean) => Promise<void>;
    addInbound: (node: any) => Promise<void>;
    updateInbound: (node: any) => Promise<void>;
    deleteInbound: (id: string) => Promise<void>;
    toggleEnable: (id: string) => Promise<void>;
    resetTraffic: (id: string) => Promise<void>;
    resetAllTraffic: () => Promise<void>;
    setInbounds: (data: Inbound[]) => void;
}

export const useInboundStore = create<InboundStore>((set, get) => ({
    inbounds: [],
    loading: false,

    fetchInbounds: async (background = false) => {
        if (!background) set({ loading: true });
        try {
            const res = await inboundApi.getInbounds();
            if (res.success) {
                const processed = (res.obj || []).map(item => ({
                    ...item,
                    settings: typeof item.settings === 'string' ? JSON.parse(item.settings) : item.settings,
                    streamSettings: typeof item.streamSettings === 'string' ? JSON.parse(item.streamSettings) : item.streamSettings,
                    sniffing: typeof item.sniffing === 'string' ? JSON.parse(item.sniffing) : item.sniffing,
                    allocate: typeof item.allocate === 'string' ? JSON.parse(item.allocate) : item.allocate,
                })).sort((a, b) => String(a.id).localeCompare(String(b.id), undefined, { numeric: true }));
                set({ inbounds: processed });
            }
        } finally {
            if (!background) set({ loading: false });
        }
    },

    addInbound: async (node: any) => {
        try {
            const res = await inboundApi.createInbound(node);
            if (res.success) {
                await get().fetchInbounds();
            } else {
                throw new Error(res.msg || 'Failed to add inbound');
            }
        } catch (error: any) {
            console.error('Failed to add inbound:', error);
            const errorMsg = error.response?.data?.msg || error.message || 'Unknown error';
            useDialogStore.getState().showAlert(errorMsg, 'Error');
            throw error;
        }
    },

    updateInbound: async (node: any) => {
        const res = await inboundApi.updateInbound(node.id, node);
        if (res.success) {
            await get().fetchInbounds();
        }
    },

    deleteInbound: async (id: string) => {
        const res = await inboundApi.deleteInbound(id);
        if (res.success) {
            set((state) => ({
                inbounds: state.inbounds.filter((item) => item.id !== id)
            }));
        }
    },

    toggleEnable: async (id: string) => {
        const node = get().inbounds.find(i => i.id === id);
        if (!node) return;
        const res = await inboundApi.toggleInbound(id, !node.enable);
        if (res.success) {
            set((state) => ({
                inbounds: state.inbounds.map((item) =>
                    item.id === id ? { ...item, enable: !item.enable } : item
                )
            }));
        }
    },

    resetTraffic: async (id: string) => {
        try {
            const res = await inboundApi.resetTraffic(id);
            if (res.success) {
                set((state) => ({
                    inbounds: state.inbounds.map((item) =>
                        item.id === id ? { ...item, up: 0, down: 0 } : item
                    )
                }));
            }
        } catch (e) {
            console.error(e);
        }
    },

    resetAllTraffic: async () => {
        try {
            const res = await inboundApi.resetAllTraffic();
            if (res.success) {
                set((state) => ({
                    inbounds: state.inbounds.map((item) => ({ ...item, up: 0, down: 0 }))
                }));
            }
        } catch (e) {
            console.error(e);
        }
    },

    setInbounds: (data) => set({ inbounds: data }),
}));