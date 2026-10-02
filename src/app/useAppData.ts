import { useCallback, useEffect, useRef, useState } from "react";
import { toast } from "react-toastify";
import { demoPayload } from "../mocks/demoPayload";
import type { BootstrapPayload } from "../shared/contracts";
import { translations } from "../shared/i18n";
import { api, isDesktop } from "../shared/tauri/client";
import { recordingNotifications } from "./recordingNotifications";

export function useAppData() {
  const [payload, setPayload] = useState<BootstrapPayload>(demoPayload);
  const [loading, setLoading] = useState(isDesktop);
  const payloadRef = useRef(payload);

  const refresh = useCallback(async () => {
    if (!isDesktop) return payloadRef.current;
    const next = await api.bootstrap();
    payloadRef.current = next;
    setPayload(next);
    return next;
  }, []);

  useEffect(() => {
    void refresh()
      .catch((reason) => toast.error(String(reason), { autoClose: false, closeOnClick: false }))
      .finally(() => setLoading(false));
    if (!isDesktop) return;
    let unlisten: (() => void) | undefined;
    void api
      .listenEngine(async () => {
        const previous = payloadRef.current;
        try {
          const next = await refresh();
          if (next.settings.notificationsEnabled) {
            const messages = recordingNotifications(
              previous,
              next,
              translations[next.settings.locale],
            );
            for (const message of messages) await api.notify("Live Downloader", message);
          }
        } catch (reason) {
          toast.error(String(reason), { autoClose: false, closeOnClick: false });
        }
      })
      .then((dispose) => {
        unlisten = dispose;
      });
    return () => unlisten?.();
  }, [refresh]);

  const updatePayload = useCallback((updates: Partial<BootstrapPayload>) => {
    setPayload((current) => {
      const next = { ...current, ...updates };
      payloadRef.current = next;
      return next;
    });
  }, []);

  return { payload, setPayload, updatePayload, loading, refresh };
}
