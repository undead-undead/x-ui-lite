import { useTranslation } from 'react-i18next';
import { useModalStore } from '../../store/useModalStore';
import { useInboundStore } from '../../store/useInboundStore';
import { useDialogStore } from '../../store/useDialogStore';
import { X } from 'lucide-react';
import { Switch } from '../ui/Switch';
import { generateUUID } from '../../utils/uuid';
import { useInboundForm } from './useInboundForm';

export const AddInboundModal = () => {
    const { t } = useTranslation();
    const { isOpen, closeModal, editingNode } = useModalStore();
    const { addInbound, updateInbound } = useInboundStore();

    const form = useInboundForm(editingNode, isOpen);

    const generateRealityKeys = async () => {
        try {
            const { generateRealityKeys: apiGenerateKeys } = await import('../../api/xray');
            const keys = await apiGenerateKeys();
            form.setRealityPrivateKey(keys.private_key);
            form.setRealityPublicKey(keys.public_key);
        } catch (error) {
            useDialogStore.getState().showAlert('Failed to generate keys, please try again', 'Error');
        }
    };

    const generateShortIds = () => {
        const randomValues = new Uint8Array(4);
        if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
            crypto.getRandomValues(randomValues);
        } else {
            for (let i = 0; i < 4; i++) {
                randomValues[i] = Math.floor(Math.random() * 256);
            }
        }
        const shortId = Array.from(randomValues)
            .map(b => b.toString(16).padStart(2, '0')).join('');
        form.setRealityShortIds(shortId);
    };

    const handleConfirm = async () => {
        if (!form.remark.trim()) {
            useDialogStore.getState().showAlert(t('inbound.modal.remark_empty'), t('common.error') || 'Error');
            return;
        }
        if (!form.port || isNaN(Number(form.port))) {
            useDialogStore.getState().showAlert(t('inbound.modal.port_error'), t('common.error') || 'Error');
            return;
        }

        let settings: any = {};

        if (form.protocol === 'vless' || form.protocol === 'vmess') {
            if (!form.uuid) {
                useDialogStore.getState().showAlert(t('inbound.modal.uuid_empty'), t('common.error') || 'Error');
                return;
            }
            // Strict UUID length check for xray-lite validation
            if (form.uuid.trim().length !== 36) {
                useDialogStore.getState().showAlert("UUID length must be 36 characters (xray-lite strict check)", t('common.error') || 'Error');
                return;
            }
            settings.clients = [{
                id: form.uuid.trim(),
                ...(form.flow && form.protocol === 'vless' && { flow: form.flow }),
                ...(form.level && { level: Number(form.level) }),
                ...(form.email && { email: form.email }),
                ...(form.protocol === 'vmess' && { alterId: Number(form.alterId) }),
            }];
            if (form.protocol === 'vless') {
                settings.decryption = form.decryption;
            }
        } else if (form.protocol === 'trojan') {
            if (!form.password) {
                useDialogStore.getState().showAlert(t('inbound.modal.password_empty'), t('common.error') || 'Error');
                return;
            }
            settings.clients = [{
                password: form.password,
                ...(form.level && { level: Number(form.level) }),
                ...(form.email && { email: form.email }),
            }];
        } else if (form.protocol === 'shadowsocks') {
            if (!form.ssPassword) {
                useDialogStore.getState().showAlert(t('inbound.modal.password_empty'), t('common.error') || 'Error');
                return;
            }
            settings = {
                method: form.ssMethod,
                password: form.ssPassword,
                network: form.ssNetwork,
            };
        }

        let streamSettings: any = {
            network: form.network,
            security: form.security,
        };

        if (form.network === 'ws') {
            streamSettings.wsSettings = {
                path: form.wsPath,
                ...(form.wsHost && { headers: { Host: form.wsHost } }),
            };
        } else if (form.network === 'grpc') {
            streamSettings.grpcSettings = {
                serviceName: form.grpcServiceName,
                multiMode: form.grpcMultiMode,
            };
        } else if (form.network === 'h2') {
            streamSettings.httpSettings = {
                ...(form.h2Host && { host: form.h2Host.split(',').map(h => h.trim()) }),
                path: form.h2Path,
            };
        } else if (form.network === 'xhttp') {
            streamSettings.xhttpSettings = {
                mode: form.xhttpMode,
                path: form.xhttpPath,
                ...(form.xhttpHost && { host: form.xhttpHost }),
            };
        }

        if (form.security === 'reality') {
            if (!form.realityPrivateKey) {
                useDialogStore.getState().showAlert(t('inbound.modal.reality_private_key_empty'), t('common.error') || 'Error');
                return;
            }
            streamSettings.realitySettings = {
                show: form.realityShow,
                dest: form.realityDest,
                xver: Number(form.realityXver),
                serverNames: form.realityServerNames.split('\n').filter(s => s.trim()),
                privateKey: form.realityPrivateKey,
                publicKey: form.realityPublicKey,
                shortIds: form.realityShortIds.split('\n').filter(s => s.trim()),
                fingerprint: form.realityFingerprint,
                ...(form.realityMinClientVer && { minClientVer: form.realityMinClientVer }),
                ...(form.realityMaxClientVer && { maxClientVer: form.realityMaxClientVer }),
                ...(form.realityMaxTimeDiff && { maxTimeDiff: Number(form.realityMaxTimeDiff) }),
            };
        }

        if (form.tcpFastOpen || form.tcpNoDelay || form.acceptProxyProtocol) {
            streamSettings.sockopt = {
                ...(form.tcpFastOpen && { tcpFastOpen: true }),
                ...(form.tcpNoDelay && { tcpNoDelay: true }),
            };
        }

        if (form.acceptProxyProtocol) {
            streamSettings.acceptProxyProtocol = true;
        }

        const data: any = {
            id: editingNode?.id || generateUUID(),
            remark: form.remark,
            enable: form.isEnable,
            port: Number(form.port),
            protocol: form.protocol,
            ...(form.tag && { tag: form.tag }),
            ...(form.listen && { listen: form.listen }),
            settings,
            streamSettings,
            total: Number(form.totalTraffic) * 1024 * 1024 * 1024,
            expiry: form.expiryTime ? new Date(form.expiryTime).getTime() : 0,
            up: editingNode?.up || 0,
            down: editingNode?.down || 0,
        };

        try {
            if (editingNode) {
                await updateInbound(data);
            } else {
                await addInbound(data);
            }
            closeModal();
        } catch (error: any) {
        }
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-100 flex items-center justify-center p-4">
            <div className="absolute inset-0 bg-black/40 backdrop-blur-sm" onClick={closeModal} />

            <div className="relative w-full max-w-4xl bg-white rounded-xl shadow-2xl flex flex-col max-h-[95vh] overflow-hidden animate-in zoom-in-95 duration-200">
                {/* Header */}
                <div className="px-6 py-4 flex items-center justify-between border-b border-gray-100 shrink-0">
                    <h2 className="text-lg font-bold text-gray-800">
                        {editingNode ? t('inbound.modal.title_edit') : t('inbound.modal.title_add')}
                    </h2>
                    <button onClick={closeModal} className="text-gray-400 hover:text-black transition-colors">
                        <X size={20} />
                    </button>
                </div>

                <div className="flex-1 overflow-y-auto px-6 py-6 space-y-4 no-scrollbar text-black">
                    <div className="space-y-4">
                        <h3 className="font-bold text-gray-700 text-sm border-b pb-2">{t('inbound.modal.base_config')}</h3>

                        <div className="grid grid-cols-12 gap-4 items-center">
                            <div className="col-span-8 flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                    <span className="text-red-500 mr-1">*</span>{t('inbound.modal.remark')}:
                                </label>
                                <input
                                    value={form.remark}
                                    onChange={(e) => form.setRemark(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 bg-white"
                                    placeholder={t('inbound.modal.remark_placeholder')}
                                />
                            </div>
                            <div className="col-span-4 flex items-center gap-3 justify-end">
                                <label className="text-sm font-bold text-gray-600">{t('inbound.modal.enable')}:</label>
                                <Switch checked={form.isEnable} onChange={form.setIsEnable} />
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                    <span className="text-red-500 mr-1">*</span>{t('inbound.modal.protocol')}:
                                </label>
                                <select
                                    value={form.protocol}
                                    onChange={(e) => form.setProtocol(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none cursor-pointer bg-white"
                                    disabled
                                >
                                    <option value="vless">VLESS</option>
                                </select>
                            </div>

                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                    <span className="text-red-500 mr-1">*</span>{t('inbound.modal.port')}:
                                </label>
                                <input
                                    value={form.port}
                                    onChange={(e) => form.setPort(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    placeholder={t('inbound.modal.port_placeholder')}
                                />
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.tag')}:</label>
                                <input
                                    value={form.tag}
                                    onChange={(e) => form.setTag(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    placeholder={t('inbound.modal.tag_placeholder')}
                                />
                            </div>

                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.listen')}:</label>
                                <input
                                    value={form.listen}
                                    onChange={(e) => form.setListen(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    placeholder={t('inbound.modal.listen_placeholder')}
                                />
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.total_traffic')}:</label>
                                <input
                                    value={form.totalTraffic}
                                    onChange={(e) => {
                                        const val = e.target.value;
                                        if (/^\d*$/.test(val)) {
                                            form.setTotalTraffic(val);
                                        }
                                    }}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    placeholder={t('inbound.modal.total_traffic_placeholder')}
                                />
                            </div>

                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.expiry_time')}:</label>
                                <input
                                    type="date"
                                    value={form.expiryTime}
                                    onChange={(e) => form.setExpiryTime(e.target.value)}
                                    className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none cursor-pointer bg-white"
                                />
                            </div>
                        </div>
                    </div>


                    <div className="space-y-4">
                        <h3 className="font-bold text-gray-700 text-sm border-b pb-2">{t('inbound.modal.protocol_config')}</h3>

                        <div className="flex items-center gap-3">
                            <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                <span className="text-red-500 mr-1">*</span>{t('inbound.modal.uuid')}:
                            </label>
                            <input
                                value={form.uuid}
                                onChange={(e) => form.setUuid(e.target.value)}
                                className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm font-mono outline-none bg-white"
                            />
                            <button
                                onClick={() => form.setUuid(generateUUID())}
                                className="px-4 py-2 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors"
                            >
                                {t('inbound.modal.generate')}
                            </button>
                        </div>

                        <div className="flex items-center gap-3">
                            <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.flow')}:</label>
                            <select
                                value={form.flow}
                                onChange={(e) => form.setFlow(e.target.value)}
                                className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                            >
                                <option value="">{t('inbound.modal.flow_none')}</option>
                                <option value="xtls-rprx-vision">xtls-rprx-vision</option>
                            </select>
                            <span className="text-xs text-red-500 font-medium shrink-0">{t('inbound.modal.flow_xhttp_tip')}</span>
                        </div>
                    </div>

                    <div className="space-y-4">
                        <h3 className="font-bold text-gray-700 text-sm border-b pb-2">{t('inbound.modal.stream_config')}</h3>

                        <div className="flex items-center gap-3">
                            <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.network')}:</label>
                            <select
                                value={form.network}
                                onChange={(e) => form.setNetwork(e.target.value)}
                                className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                            >
                                <option value="tcp">TCP</option>
                                <option value="xhttp">XHTTP</option>
                            </select>
                        </div>

                        {form.network === 'xhttp' && (
                            <>
                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.xhttp_mode')}:</label>
                                    <select
                                        value={form.xhttpMode}
                                        onChange={(e) => form.setXhttpMode(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    >
                                        <option value="auto">{t('inbound.modal.xhttp_mode_auto')}</option>
                                        <option value="packet-up">{t('inbound.modal.xhttp_mode_packet')}</option>
                                        <option value="stream-up">{t('inbound.modal.xhttp_mode_stream')}</option>
                                        <option value="stream-one">{t('inbound.modal.xhttp_mode_one')}</option>
                                    </select>
                                    <span className="text-xs text-gray-500 shrink-0">{t('common.default')}: auto</span>
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.xhttp_path')}:</label>
                                    <input
                                        value={form.xhttpPath}
                                        onChange={(e) => form.setXhttpPath(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                        placeholder={t('inbound.modal.xhttp_path_placeholder')}
                                    />
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.xhttp_host')}:</label>
                                    <input
                                        value={form.xhttpHost}
                                        onChange={(e) => form.setXhttpHost(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                        placeholder={t('inbound.modal.xhttp_host_placeholder')}
                                    />
                                    <span className="text-xs text-gray-500 shrink-0">{t('inbound.modal.optional')}</span>
                                </div>
                            </>
                        )}
                    </div>

                    <div className="space-y-4">
                        <h3 className="font-bold text-gray-700 text-sm border-b pb-2">{t('inbound.modal.security_config')}</h3>

                        <div className="flex items-center gap-3">
                            <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.security')}:</label>
                            <select
                                value={form.security}
                                onChange={(e) => {
                                    form.setSecurity(e.target.value);
                                    if (e.target.value === 'reality') {
                                        if (!form.realityShortIds) {
                                            generateShortIds();
                                        }
                                        if (!form.realityPrivateKey) {
                                            generateRealityKeys();
                                        }
                                    }
                                }}
                                className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                            >
                                <option value="none">{t('inbound.modal.security_none')}</option>
                                <option value="reality">Reality</option>
                            </select>
                        </div>



                        {form.security === 'reality' && (
                            <>
                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                        <span className="text-red-500 mr-1">*</span>{t('inbound.modal.reality_dest')}:
                                    </label>
                                    <input
                                        value={form.realityDest}
                                        onChange={(e) => form.setRealityDest(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                        placeholder={t('inbound.modal.reality_dest_placeholder')}
                                    />
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.reality_server_names')}:</label>
                                    <textarea
                                        value={form.realityServerNames}
                                        onChange={(e) => form.setRealityServerNames(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white resize-none"
                                        rows={2}
                                        placeholder={t('inbound.modal.reality_server_names_placeholder')}
                                    />
                                </div>


                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.reality_fingerprint')}:</label>
                                    <select
                                        value={form.realityFingerprint}
                                        onChange={(e) => form.setRealityFingerprint(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm outline-none bg-white"
                                    >
                                        <option value="chrome">Chrome</option>
                                        <option value="firefox">Firefox</option>
                                        <option value="safari">Safari</option>
                                        <option value="edge">Edge</option>
                                    </select>
                                </div>


                                <div className="flex gap-2">
                                    <button
                                        onClick={generateRealityKeys}
                                        className="flex-1 px-4 py-2 bg-blue-500 text-white rounded-md text-sm hover:bg-blue-600 transition-colors"
                                    >
                                        {t('inbound.modal.reality_keys_btn')}
                                    </button>
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">
                                        <span className="text-red-500 mr-1">*</span>{t('inbound.modal.reality_private_key')}:
                                    </label>
                                    <input
                                        value={form.realityPrivateKey}
                                        onChange={(e) => form.setRealityPrivateKey(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm font-mono outline-none bg-white"
                                    />
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.reality_public_key')}:</label>
                                    <input
                                        value={form.realityPublicKey}
                                        onChange={(e) => form.setRealityPublicKey(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm font-mono outline-none bg-white"
                                        readOnly
                                    />
                                </div>

                                <div className="flex items-center gap-3">
                                    <label className="text-sm font-bold text-gray-600 w-24 text-right shrink-0">{t('inbound.modal.reality_short_ids')}:</label>
                                    <textarea
                                        value={form.realityShortIds}
                                        onChange={(e) => form.setRealityShortIds(e.target.value)}
                                        className="flex-1 border border-gray-200 rounded-md px-3 py-2 text-sm font-mono outline-none bg-white resize-none"
                                        rows={2}
                                        placeholder={t('inbound.modal.reality_short_ids_placeholder')}
                                    />
                                    <button
                                        onClick={generateShortIds}
                                        className="px-4 py-2 bg-green-500 text-white rounded-md text-sm hover:bg-green-600 transition-colors shrink-0"
                                    >
                                        {t('inbound.modal.generate')}
                                    </button>
                                </div>
                            </>
                        )}
                    </div>




                    <div className="space-y-4">
                        <h3 className="font-bold text-gray-700 text-sm border-b pb-2">{t('inbound.modal.socket_config')}</h3>

                        <div className="grid grid-cols-2 gap-4">
                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-40 text-right shrink-0">Accept Proxy Protocol:</label>
                                <Switch checked={form.acceptProxyProtocol} onChange={form.setAcceptProxyProtocol} />
                            </div>

                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-40 text-right shrink-0">TCP Fast Open:</label>
                                <Switch checked={form.tcpFastOpen} onChange={form.setTcpFastOpen} />
                            </div>

                            <div className="flex items-center gap-3">
                                <label className="text-sm font-bold text-gray-600 w-40 text-right shrink-0">TCP No Delay:</label>
                                <Switch checked={form.tcpNoDelay} onChange={form.setTcpNoDelay} />
                            </div>
                        </div>
                    </div>
                </div>

                {/* Footer */}
                <div className="px-6 py-4 border-t border-gray-50 flex items-center justify-end gap-3 shrink-0">
                    <button
                        onClick={closeModal}
                        className="px-6 py-2 bg-white text-black rounded-lg text-sm font-bold border border-black hover:bg-gray-50 transition-colors"
                    >
                        {t('inbound.modal.cancel')}
                    </button>
                    <button
                        onClick={handleConfirm}
                        className="px-6 py-2 bg-blue-600 text-white rounded-lg text-sm font-bold hover:bg-blue-700 transition-colors"
                    >
                        {editingNode ? t('common.save') : t('common.add')}
                    </button>
                </div>
            </div>
        </div>
    );
};
