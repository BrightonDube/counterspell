import { afterEach, describe, expect, it, vi } from 'vitest';
import { AuthApiError } from './client.js';
import { signInWithPasskey, verifyTwoFactor } from './security';

afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

describe('sign-in security', () => {
  it('passes the authenticator or recovery code and uses the server destination', async () => {
    const fetch = vi
      .fn()
      .mockResolvedValue(Response.json({ return_to: '/problems' }));
    vi.stubGlobal('fetch', fetch);

    await expect(verifyTwoFactor('4AB1-CD29')).resolves.toEqual({
      return_to: '/problems'
    });
    expect(fetch).toHaveBeenCalledWith('/api/auth/totp/verify', {
      method: 'POST',
      credentials: 'same-origin',
      headers: {
        accept: 'application/json',
        'content-type': 'application/json'
      },
      body: '{"code":"4AB1-CD29"}'
    });
  });

  it('reports an expired challenge distinctly from an incorrect code', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          Response.json({ code: 'security_challenge_expired' }, { status: 401 })
        )
    );

    const failure = await verifyTwoFactor('123456').catch((error) => error);
    expect(failure).toBeInstanceOf(AuthApiError);
    expect(failure.code).toBe('security_challenge_expired');
  });

  it('cancels a pending passkey prompt after 60 seconds', async () => {
    vi.useFakeTimers();
    vi.stubGlobal('PublicKeyCredential', {});
    const get = vi.fn(
      ({ signal }: CredentialRequestOptions) =>
        new Promise<Credential | null>((_, reject) => {
          signal?.addEventListener('abort', () => reject(signal.reason), {
            once: true
          });
        })
    );
    vi.stubGlobal('navigator', { credentials: { get } });
    const fetch = vi.fn().mockResolvedValue(
      Response.json({
        flow_id: 'challenge-flow',
        publicKey: { challenge: '_w', timeout: 300_000 }
      })
    );
    vi.stubGlobal('fetch', fetch);

    const attempt = signInWithPasskey('/problems').catch(
      (failure: unknown) => failure
    );
    await vi.advanceTimersByTimeAsync(59_999);
    const options = get.mock.calls[0][0];
    expect(options.publicKey?.timeout).toBe(60_000);
    expect(options.signal?.aborted).toBe(false);

    await vi.advanceTimersByTimeAsync(1);
    await expect(attempt).resolves.toMatchObject({ name: 'AbortError' });
    expect(fetch).toHaveBeenCalledTimes(1);
  });

  it('decodes and serializes the passkey assertion on browsers without WebAuthn JSON helpers', async () => {
    class BrowserCredential {
      type = 'public-key';
      id = '-_8';
      rawId = Uint8Array.of(251, 255).buffer;
      response = {
        clientDataJSON: Uint8Array.of(1).buffer,
        authenticatorData: Uint8Array.of(2).buffer,
        signature: Uint8Array.of(3).buffer,
        userHandle: Uint8Array.of(4).buffer
      };
      getClientExtensionResults() {
        return {};
      }
    }
    vi.stubGlobal('PublicKeyCredential', BrowserCredential);
    vi.stubGlobal('isSecureContext', true);
    const get = vi.fn().mockResolvedValue(new BrowserCredential());
    vi.stubGlobal('navigator', { credentials: { get } });
    const fetch = vi
      .fn()
      .mockResolvedValueOnce(
        Response.json({
          flow_id: 'challenge-flow',
          publicKey: {
            challenge: '_w',
            allowCredentials: [{ type: 'public-key', id: '-_8' }]
          }
        })
      )
      .mockResolvedValueOnce(Response.json({ return_to: '/problems' }));
    vi.stubGlobal('fetch', fetch);

    await expect(signInWithPasskey('/problems')).resolves.toEqual({
      return_to: '/problems'
    });
    const options = get.mock.calls[0][0].publicKey;
    expect(new Uint8Array(options.challenge)).toEqual(Uint8Array.of(255));
    expect(new Uint8Array(options.allowCredentials[0].id)).toEqual(
      Uint8Array.of(251, 255)
    );
    expect(fetch.mock.calls[1][0]).toBe('/api/auth/passkeys/login/finish');
    expect(JSON.parse(fetch.mock.calls[1][1].body)).toMatchObject({
      flow_id: 'challenge-flow',
      credential: {
        id: '-_8',
        rawId: '-_8',
        type: 'public-key',
        response: {
          clientDataJSON: 'AQ',
          authenticatorData: 'Ag',
          signature: 'Aw',
          userHandle: 'BA'
        }
      }
    });
  });
});
