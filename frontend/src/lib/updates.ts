import { invoke } from '@tauri-apps/api/core';

export type UpdatePlatform = 'desktop' | 'android' | 'web';

export interface UpdateStatus {
  platform: UpdatePlatform;
  available: boolean;
  version?: string;
  notes?: string;
  canInstallInApp: boolean;
  externalUrl?: string;
  error?: string;
}

export interface CheckForUpdateOptions {
  manual?: boolean;
}

interface AndroidStoreUpdateStatus {
  available: boolean;
  version?: string;
  notes?: string;
  canInstallInApp: boolean;
  externalUrl: string;
}

interface DesktopUpdate {
  version: string;
  date?: string;
  body?: string;
  downloadAndInstall(onEvent?: (event: unknown) => void): Promise<void>;
}

export interface UpdateServiceAdapters {
  isTauri(): boolean;
  isAndroid(): boolean;
  desktopCheck(): Promise<DesktopUpdate | null>;
  relaunch(): Promise<void>;
  invoke<T>(command: string): Promise<T>;
  openExternal(url: string): Promise<void>;
  registerWebUpdater(onNeedRefresh: () => void): Promise<WebUpdateHandle | null>;
  getDismissedVersion(): string | null;
  setDismissedVersion(version: string): void;
}

interface WebUpdateHandle {
  update(reloadPage?: boolean): Promise<void>;
  check(): Promise<void>;
}

const DISMISSED_UPDATE_KEY = 'memory-pak.dismissed-update';
const ANDROID_STORE_URL = 'https://play.google.com/store/apps/details?id=com.Aspenini.MemoryPak';

export interface UpdateService {
  checkForUpdate(options?: CheckForUpdateOptions): Promise<UpdateStatus>;
  installUpdate(): Promise<void>;
  dismissUpdate(version: string): void;
}

export function createUpdateService(
  onStatus?: (status: UpdateStatus) => void,
  adapters: UpdateServiceAdapters = defaultAdapters()
): UpdateService {
  const platform = resolvePlatform(adapters);
  let pendingDesktopUpdate: DesktopUpdate | null = null;
  let webUpdate: WebUpdateHandle | null = null;
  let webUpdateReady = false;

  const publish = (status: UpdateStatus, manual?: boolean): UpdateStatus => {
    const dismissed = status.version && adapters.getDismissedVersion() === status.version;
    const next = dismissed && !manual ? { ...status, available: false } : status;
    onStatus?.(next);
    return next;
  };

  const service: UpdateService = {
    async checkForUpdate(options: CheckForUpdateOptions = {}) {
      const manual = Boolean(options.manual);
      try {
        if (platform === 'desktop') {
          pendingDesktopUpdate = await adapters.desktopCheck();
          if (!pendingDesktopUpdate) {
            return publish(baseStatus('desktop'), manual);
          }
          return publish(
            {
              platform: 'desktop',
              available: true,
              version: pendingDesktopUpdate.version,
              notes: pendingDesktopUpdate.body,
              canInstallInApp: true
            },
            manual
          );
        }

        if (platform === 'android') {
          const update = await adapters.invoke<AndroidStoreUpdateStatus>(
            'android_check_store_update'
          );
          const available = update.available || (manual && Boolean(update.externalUrl));
          return publish(
            {
              platform: 'android',
              available,
              version: update.version,
              notes: update.notes,
              canInstallInApp: update.canInstallInApp,
              externalUrl: update.externalUrl || ANDROID_STORE_URL
            },
            manual
          );
        }

        if (!webUpdate) {
          webUpdate = await adapters.registerWebUpdater(() => {
            webUpdateReady = true;
            publish({
              platform: 'web',
              available: true,
              version: 'web',
              notes: 'A new web build is ready.',
              canInstallInApp: true
            });
          });
        }
        await webUpdate?.check();
        return publish(
          {
            platform: 'web',
            available: webUpdateReady,
            version: webUpdateReady ? 'web' : undefined,
            notes: webUpdateReady ? 'A new web build is ready.' : undefined,
            canInstallInApp: Boolean(webUpdate)
          },
          manual
        );
      } catch (cause) {
        if (!manual) {
          return publish(baseStatus(platform), manual);
        }
        return publish(
          {
            ...baseStatus(platform),
            error: cause instanceof Error ? cause.message : String(cause)
          },
          manual
        );
      }
    },

    async installUpdate() {
      if (platform === 'desktop') {
        if (!pendingDesktopUpdate) {
          pendingDesktopUpdate = await adapters.desktopCheck();
        }
        if (!pendingDesktopUpdate) return;
        await pendingDesktopUpdate.downloadAndInstall();
        await adapters.relaunch();
        return;
      }

      if (platform === 'android') {
        const target = await adapters
          .invoke<string>('android_open_update_target')
          .catch(() => ANDROID_STORE_URL);
        await adapters.invoke('android_start_store_update').catch(() => undefined);
        await adapters.openExternal(target || ANDROID_STORE_URL);
        return;
      }

      await webUpdate?.update(true);
    },

    dismissUpdate(version: string) {
      adapters.setDismissedVersion(version);
    }
  };

  return service;
}

function baseStatus(platform: UpdatePlatform): UpdateStatus {
  return { platform, available: false, canInstallInApp: false };
}

function resolvePlatform(adapters: UpdateServiceAdapters): UpdatePlatform {
  if (!adapters.isTauri()) return 'web';
  return adapters.isAndroid() ? 'android' : 'desktop';
}

function defaultAdapters(): UpdateServiceAdapters {
  return {
    isTauri: () => typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__),
    isAndroid: () =>
      typeof navigator !== 'undefined' && /Android/i.test(navigator.userAgent || ''),
    desktopCheck: async () => {
      const { check } = await import('@tauri-apps/plugin-updater');
      return (await check()) as DesktopUpdate | null;
    },
    relaunch: async () => {
      const { relaunch } = await import('@tauri-apps/plugin-process');
      await relaunch();
    },
    invoke,
    openExternal: async (url: string) => {
      if (typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
        const { openUrl } = await import('@tauri-apps/plugin-opener');
        await openUrl(url);
        return;
      }
      window.open(url, '_blank', 'noopener,noreferrer');
    },
    registerWebUpdater: async (onNeedRefresh: () => void) => {
      if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return null;
      const { registerSW } = await import('virtual:pwa-register');
      const update = registerSW({
        immediate: true,
        onNeedRefresh
      });
      return {
        update,
        check: async () => {
          const registrations = await navigator.serviceWorker.getRegistrations();
          await Promise.all(registrations.map((registration) => registration.update()));
        }
      };
    },
    getDismissedVersion: () => localStorage.getItem(DISMISSED_UPDATE_KEY),
    setDismissedVersion: (version: string) => localStorage.setItem(DISMISSED_UPDATE_KEY, version)
  };
}
