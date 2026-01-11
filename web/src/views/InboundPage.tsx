import { useState, useMemo, useEffect } from 'react';
import { useInboundStore } from '../store/useInboundStore';
import { InboundTable } from '../components/InboundTable';
import { InboundTableVirtual } from '../components/InboundTableVirtual';
import { Plus, CheckCircle2, XCircle, Database } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useModalStore } from '../store/useModalStore';
import { useBackupModalStore } from '../store/useBackupModalStore';
import { useDialogStore } from '../store/useDialogStore';
import { toast } from 'react-hot-toast';
import { formatTraffic } from '../utils/format';
import { useDebouncedValue } from '@mantine/hooks';
import { checkRealityDomain, quickCheckRealityDomainSync, type DomainCheckResult } from '../utils/realityDomainChecker';

const VIRTUAL_SCROLL_THRESHOLD = 50;

export const InboundPage = () => {
    const { t } = useTranslation();

    const inbounds = useInboundStore((state) => state.inbounds);
    const fetchInbounds = useInboundStore((state) => state.fetchInbounds);
    const openModal = useModalStore((state) => state.openModal);
    const openBackupModal = useBackupModalStore((state) => state.open);

    const resetAllTraffic = useInboundStore((state) => state.resetAllTraffic);

    const [searchQuery, setSearchQuery] = useState('');
    const [realityDomain, setRealityDomain] = useState('');
    const [isChecking, setIsChecking] = useState(false);
    const [checkResult, setCheckResult] = useState<DomainCheckResult | null>(null);

    const currentTotals = useMemo(() => {
        return inbounds.reduce(
            (acc, inbound) => ({
                upload: acc.upload + (inbound.up || 0),
                download: acc.download + (inbound.down || 0),
            }),
            { upload: 0, download: 0 }
        );
    }, [inbounds]);

    const { totalUpload, totalDownload } = useMemo(() => {
        return {
            totalUpload: formatTraffic(currentTotals.upload),
            totalDownload: formatTraffic(currentTotals.download),
        };
    }, [currentTotals]);

    const handleResetTrafficStats = () => {
        useDialogStore.getState().showConfirm(
            t('inbound.confirm.reset_msg'),
            async () => {
                await resetAllTraffic();
                toast.success(t('inbound.confirm.reset_success'));
            },
            t('inbound.confirm.reset_title')
        );
    };

    useEffect(() => {
        fetchInbounds();

        const timer = setInterval(() => {
            fetchInbounds(true);
        }, 5000);

        return () => clearInterval(timer);
    }, [fetchInbounds]);

    const [debouncedQuery] = useDebouncedValue(searchQuery, 300);

    const filteredInbounds = useMemo(() => {
        if (!debouncedQuery) return inbounds;

        const query = debouncedQuery.toLowerCase();
        return inbounds.filter(inbound =>
            inbound.remark.toLowerCase().includes(query) ||
            inbound.protocol.toLowerCase().includes(query) ||
            inbound.port.toString().includes(query)
        );
    }, [inbounds, debouncedQuery]);


    const quickResult = useMemo(() => {
        if (!realityDomain.trim()) {
            return null;
        }
        return quickCheckRealityDomainSync(realityDomain);
    }, [realityDomain]);

    const handleCheckDomain = async () => {
        if (!realityDomain.trim()) return;

        setIsChecking(true);
        try {
            const result = await checkRealityDomain(realityDomain);
            setCheckResult(result);
        } catch (error) {
            setCheckResult({
                isValid: false,
                message: t('inbound.check_failed'),
                details: t('inbound.check_network_error'),
            });
        } finally {
            setIsChecking(false);
        }
    };

    useMemo(() => {
        setCheckResult(null);
    }, [realityDomain]);

    const displayResult = checkResult || quickResult;

    const useVirtualScroll = filteredInbounds.length > VIRTUAL_SCROLL_THRESHOLD;
    const TableComponent = useVirtualScroll ? InboundTableVirtual : InboundTable;

    return (
        <div className="flex-1 min-h-screen bg-gray-50 p-8 lg:p-14 overflow-y-auto relative font-sans">
            <div className="max-w-full relative z-10">
                <div className="bg-white border border-gray-200 rounded-2xl shadow-lg animate-in fade-in slide-in-from-bottom-6 duration-1000">
                    <div className="flex items-center justify-between gap-4 px-8 py-6 border-b border-gray-200">
                        <div className="flex items-center gap-4">
                            <button
                                onClick={() => openModal()}
                                className="flex items-center justify-center px-5 py-1.5 bg-white text-black rounded-xl text-[13px] font-bold border border-black hover:-translate-y-[2px] hover:shadow-[0_4px_0_0_#94a3b8] active:translate-y-px active:shadow-none transition-all shadow-[0_1px_0_0_#94a3b8] whitespace-nowrap leading-none"
                                style={{ padding: '5px 24px 4px 24px' }}
                            >
                                <Plus size={16} strokeWidth={3} className="mr-2" />
                                <span>{t('inbound.modal.title_add')}</span>
                            </button>

                            <input
                                type="text"
                                value={searchQuery}
                                onChange={(e) => setSearchQuery(e.target.value)}
                                placeholder={t('inbound.search_placeholder')}
                                className="w-64 h-11 px-5 bg-white border border-black rounded-xl outline-none focus:ring-0 transition-all text-black placeholder:text-gray-300 font-bold text-[13px] shadow-[0_1px_0_0_#94a3b8]"
                            />
                        </div>

                        <div className="flex-1 flex justify-center">
                            <div className="flex items-center gap-3 w-auto min-w-fit h-11 px-5 bg-white border border-black rounded-xl shadow-[0_1px_0_0_#94a3b8]">
                                <div className="flex flex-row items-center gap-2 whitespace-nowrap">
                                    <span className="text-[10px] text-gray-500 font-medium">{t('inbound.up_total')}</span>
                                    <span className="text-[13px] font-bold text-gray-700 tabular-nums">{totalUpload}</span>
                                </div>
                                <div className="w-px h-6 bg-gray-300"></div>
                                <div className="flex flex-row items-center gap-2 whitespace-nowrap">
                                    <span className="text-[10px] text-gray-500 font-medium">{t('inbound.down_total')}</span>
                                    <span className="text-[13px] font-bold text-gray-700 tabular-nums">{totalDownload}</span>
                                </div>
                                <button
                                    onClick={handleResetTrafficStats}
                                    className="px-5 py-1.5 bg-white text-black rounded-xl text-[13px] font-bold border border-black hover:-translate-y-[2px] hover:shadow-[0_4px_0_0_#94a3b8] active:translate-y-px active:shadow-none transition-all shadow-[0_1px_0_0_#94a3b8] whitespace-nowrap leading-none"
                                    style={{ padding: '5px 24px 4px 24px' }}
                                    title={t('inbound.reset')}
                                >
                                    {t('inbound.reset')}
                                </button>
                            </div>
                        </div>

                        <div className="flex items-center gap-2">
                            <div className="relative flex items-center">
                                <input
                                    type="text"
                                    value={realityDomain}
                                    onChange={(e) => setRealityDomain(e.target.value)}
                                    onKeyDown={(e) => e.key === 'Enter' && handleCheckDomain()}
                                    placeholder={t('inbound.reality_check_placeholder')}
                                    className="w-72 h-11 px-5 pr-[120px] bg-white border border-black rounded-xl outline-none focus:ring-0 transition-all text-black placeholder:text-gray-300 font-bold text-[13px] shadow-[0_1px_0_0_#94a3b8]"
                                />

                                <button
                                    onClick={handleCheckDomain}
                                    disabled={!realityDomain.trim() || isChecking}
                                    className="absolute right-3 top-[10px] bg-white text-black border border-black rounded-xl text-[13px] font-bold hover:-translate-y-[2px] hover:shadow-[0_4px_0_0_#94a3b8] active:translate-y-px active:shadow-none disabled:cursor-not-allowed disabled:hover:translate-y-0 disabled:hover:shadow-[0_1px_0_0_#94a3b8] transition-all shadow-[0_1px_0_0_#94a3b8] flex items-center leading-none"
                                    style={{ padding: '5px 24px 4px 24px' }}
                                >
                                    {isChecking ? t('inbound.checking') : t('inbound.check')}
                                </button>
                                {displayResult && !isChecking && (
                                    <div className="absolute right-[100px] top-1/2 -translate-y-1/2">
                                        {displayResult.isValid ? (
                                            <CheckCircle2 size={18} className="text-black" />
                                        ) : (
                                            <XCircle size={18} className="text-gray-500" />
                                        )}
                                    </div>
                                )}
                                {displayResult && !isChecking && (
                                    <div className={`absolute top-full left-0 right-0 mt-2 p-3 rounded-lg text-xs font-medium shadow-lg z-50 ${displayResult.warning
                                        ? 'bg-gray-100 border border-gray-400 text-gray-800'
                                        : displayResult.isValid
                                            ? 'bg-white border border-black text-black'
                                            : 'bg-gray-100 border border-gray-400 text-gray-700'
                                        }`}>
                                        <div className="font-bold mb-1">{displayResult.message}</div>
                                        {displayResult.details && (
                                            <div className="text-xs opacity-80 mb-2">{displayResult.details}</div>
                                        )}
                                        {displayResult.score !== undefined && (
                                            <div className="text-xs font-bold mb-2">{t('inbound.score')}: {displayResult.score}/100</div>
                                        )}
                                        {displayResult.warning && (
                                            <div className="mt-2 pt-2 border-t border-current/20 whitespace-pre-line text-xs leading-relaxed">
                                                {displayResult.warning}
                                            </div>
                                        )}
                                    </div>
                                )}
                            </div>

                            <div className="flex items-center gap-6">
                                <button
                                    onClick={() => openBackupModal()}
                                    className="flex items-center justify-center px-5 py-1.5 bg-white text-black rounded-xl text-[13px] font-bold border border-black hover:-translate-y-[2px] hover:shadow-[0_4px_0_0_#94a3b8] active:translate-y-px active:shadow-none transition-all shadow-[0_1px_0_0_#94a3b8] whitespace-nowrap leading-none"
                                    style={{ padding: '5px 24px 4px 24px' }}
                                >
                                    <Database size={16} strokeWidth={2.5} className="mr-2" />
                                    <span>{t('settings.backup.manage_btn')}</span>
                                </button>
                            </div>
                        </div>
                    </div>

                    <TableComponent inbounds={filteredInbounds} isEmbedded={true} />
                </div>
            </div>
        </div>
    );
};