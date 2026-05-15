import { describe, expect, it, vi } from 'vitest';
import { createUpdateService, type UpdateServiceAdapters, type UpdateStatus } from './updates';

function adapters(overrides: Partial<UpdateServiceAdapters> = {}): UpdateServiceAdapters {
  let dismissed: string | null = null;
  const invoke: UpdateServiceAdapters['invoke'] = async <T>() => undefined as T;
  return {
    isTauri: () => false,
    isAndroid: () => false,
    desktopCheck: vi.fn(async () => null),
    relaunch: vi.fn(async () => undefined),
    invoke,
    openExternal: vi.fn(async () => undefined),
    registerWebUpdater: vi.fn(async () => null),
    getDismissedVersion: () => dismissed,
    setDismissedVersion: (version: string) => {
      dismissed = version;
    },
    ...overrides
  };
}

describe('update service', () => {
  it('reports desktop updates and installs them', async () => {
    const downloadAndInstall = vi.fn(async () => undefined);
    const relaunch = vi.fn(async () => undefined);
    const service = createUpdateService(undefined, adapters({
      isTauri: () => true,
      desktopCheck: vi.fn(async () => ({
        version: '0.4.0',
        body: 'Update notes',
        downloadAndInstall
      })),
      relaunch
    }));

    const status = await service.checkForUpdate();
    expect(status).toMatchObject({
      platform: 'desktop',
      available: true,
      version: '0.4.0',
      canInstallInApp: true
    });

    await service.installUpdate();
    expect(downloadAndInstall).toHaveBeenCalledOnce();
    expect(relaunch).toHaveBeenCalledOnce();
  });

  it('suppresses dismissed non-manual updates', async () => {
    const fake = adapters({
      isTauri: () => true,
      desktopCheck: vi.fn(async () => ({
        version: '0.4.0',
        downloadAndInstall: vi.fn(async () => undefined)
      }))
    });
    const service = createUpdateService(undefined, fake);

    service.dismissUpdate('0.4.0');
    expect(await service.checkForUpdate()).toMatchObject({ available: false });
    expect(await service.checkForUpdate({ manual: true })).toMatchObject({ available: true });
  });

  it('publishes web service worker updates from the registration callback', async () => {
    let trigger: (() => void) | undefined;
    const seen: UpdateStatus[] = [];
    const service = createUpdateService(
      (status) => seen.push(status),
      adapters({
        registerWebUpdater: vi.fn(async (onNeedRefresh) => {
          trigger = onNeedRefresh;
          return {
            update: vi.fn(async () => undefined),
            check: vi.fn(async () => undefined)
          };
        })
      })
    );

    await service.checkForUpdate();
    trigger?.();

    expect(seen[seen.length - 1]).toMatchObject({
      platform: 'web',
      available: true,
      canInstallInApp: true
    });
  });

  it('opens the store path for Android updates', async () => {
    const openExternal = vi.fn(async () => undefined);
    const invokeCalls: string[] = [];
    const storeUrl = 'https://play.google.com/store/apps/details?id=com.Aspenini.MemoryPak';
    const invoke: UpdateServiceAdapters['invoke'] = async <T>(command: string) => {
      invokeCalls.push(command);
      if (command === 'android_open_update_target') return storeUrl as T;
      return {
        available: false,
        canInstallInApp: false,
        externalUrl: storeUrl
      } as T;
    };
    const service = createUpdateService(undefined, adapters({
      isTauri: () => true,
      isAndroid: () => true,
      invoke,
      openExternal
    }));

    await expect(service.checkForUpdate({ manual: true })).resolves.toMatchObject({
      available: true,
      canInstallInApp: false,
      externalUrl: storeUrl
    });
    await service.installUpdate();

    expect(invokeCalls).toContain('android_open_update_target');
    expect(invokeCalls).toContain('android_start_store_update');
    expect(openExternal).toHaveBeenCalledWith(storeUrl);
  });
});
