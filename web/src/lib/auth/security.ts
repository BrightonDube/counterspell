import { AuthApiError, request } from './client.js';
import { messageFor } from './messages.js';

type Passkey = { id: string; name: string; created_at: string };
type SecuritySettings = {
  totp_enabled: boolean;
  can_replace_authenticator: boolean;
  passkeys: Passkey[];
};
type TotpSetup = { secret: string; uri: string };
type Destination = { return_to: string };
type SignInResult = Partial<Destination> & { two_factor_required?: boolean };
type PasskeyFlow<Options> = { flow_id: string; publicKey: Options };

const PASSKEY_SIGN_IN_TIMEOUT_MS = 60_000;

function post<T>(path: string, body: unknown): Promise<T> {
  return request(`/api/auth/${path}`, { method: 'POST', body });
}

export const getSecurity = (): Promise<SecuritySettings> =>
  request('/api/auth/security');
export const startTotp = () => post<TotpSetup>('totp/start', {});
export const enableTotp = (code: string) =>
  post<{ recovery_codes: string[] }>('totp/enable', { code });
export const disableTotp = (code: string) =>
  post<null>('totp/disable', { code });
export const verifyTwoFactor = (code: string) =>
  post<Destination>('totp/verify', { code });
export const signInWithPassword = (
  email: string,
  password: string,
  returnTo: string
) => post<SignInResult>('login', { email, password, return_to: returnTo });
export const deletePasskey = (id: string) =>
  request(`/api/auth/passkeys/${encodeURIComponent(id)}`, { method: 'DELETE' });

export const supportsPasskeys = () =>
  typeof PublicKeyCredential !== 'undefined' &&
  globalThis.isSecureContext &&
  !!navigator.credentials;

// WebAuthn Level 3 supplies JSON helpers. Earlier browsers require conversion of
// the server's base64url challenges, IDs and credential responses.
function decode(value: string) {
  return Uint8Array.from(
    atob(value.replaceAll('-', '+').replaceAll('_', '/')),
    (char) => char.charCodeAt(0)
  );
}

function encode(buffer: ArrayBuffer) {
  return btoa(
    Array.from(new Uint8Array(buffer), (byte) =>
      String.fromCharCode(byte)
    ).join('')
  )
    .replaceAll('+', '-')
    .replaceAll('/', '_')
    .replace(/=+$/, '');
}

function decodeDescriptors(
  descriptors: PublicKeyCredentialDescriptorJSON[] | undefined
) {
  return descriptors?.map(({ id, transports }) => ({
    type: 'public-key' as const,
    id: decode(id),
    transports: transports as AuthenticatorTransport[] | undefined
  }));
}

function creationOptions(
  options: PublicKeyCredentialCreationOptionsJSON
): PublicKeyCredentialCreationOptions {
  if (PublicKeyCredential.parseCreationOptionsFromJSON) {
    return PublicKeyCredential.parseCreationOptionsFromJSON(options);
  }
  return {
    ...options,
    attestation: options.attestation as
      AttestationConveyancePreference | undefined,
    challenge: decode(options.challenge),
    user: { ...options.user, id: decode(options.user.id) },
    excludeCredentials: decodeDescriptors(options.excludeCredentials),
    extensions: { credProps: options.extensions?.credProps }
  };
}

function requestOptions(
  options: PublicKeyCredentialRequestOptionsJSON
): PublicKeyCredentialRequestOptions {
  if (PublicKeyCredential.parseRequestOptionsFromJSON) {
    return PublicKeyCredential.parseRequestOptionsFromJSON(options);
  }
  return {
    ...options,
    challenge: decode(options.challenge),
    allowCredentials: decodeDescriptors(options.allowCredentials),
    extensions: { appid: options.extensions?.appid },
    userVerification: options.userVerification as
      UserVerificationRequirement | undefined
  };
}

function serialize(credential: Credential | null) {
  if (!(credential instanceof PublicKeyCredential)) {
    throw new DOMException('Passkey prompt closed', 'NotAllowedError');
  }
  if (credential.toJSON) return credential.toJSON();

  const response = credential.response as
    AuthenticatorAttestationResponse | AuthenticatorAssertionResponse;
  const clientDataJSON = encode(response.clientDataJSON);
  return {
    id: credential.id,
    rawId: encode(credential.rawId),
    type: credential.type,
    authenticatorAttachment: credential.authenticatorAttachment,
    clientExtensionResults: credential.getClientExtensionResults(),
    response:
      'attestationObject' in response
        ? {
            clientDataJSON,
            attestationObject: encode(response.attestationObject),
            transports: response.getTransports?.() ?? []
          }
        : {
            clientDataJSON,
            authenticatorData: encode(response.authenticatorData),
            signature: encode(response.signature),
            userHandle: response.userHandle ? encode(response.userHandle) : null
          }
  };
}

export async function registerPasskey(name: string) {
  const { flow_id, publicKey } = await post<
    PasskeyFlow<PublicKeyCredentialCreationOptionsJSON>
  >('passkeys/register/start', { name });
  const credential = await navigator.credentials.create({
    publicKey: creationOptions(publicKey)
  });
  await post('passkeys/register/finish', {
    flow_id,
    credential: serialize(credential)
  });
}

export async function signInWithPasskey(returnTo: string) {
  const { flow_id, publicKey } = await post<
    PasskeyFlow<PublicKeyCredentialRequestOptionsJSON>
  >('passkeys/login/start', { return_to: returnTo });
  const options = requestOptions(publicKey);
  const timeout = Math.min(
    options.timeout ?? PASSKEY_SIGN_IN_TIMEOUT_MS,
    PASSKEY_SIGN_IN_TIMEOUT_MS
  );
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);
  try {
    const credential = await navigator.credentials.get({
      publicKey: { ...options, timeout },
      signal: controller.signal
    });
    return post<SignInResult>('passkeys/login/finish', {
      flow_id,
      credential: serialize(credential)
    });
  } finally {
    clearTimeout(timer);
  }
}

export function authProblem(failure: unknown) {
  if (failure instanceof AuthApiError) return messageFor(failure.code);
  if (failure instanceof DOMException) {
    if (failure.name === 'NotAllowedError' || failure.name === 'AbortError') {
      return 'The passkey prompt closed. Try again when you’re ready.';
    }
    if (failure.name === 'InvalidStateError') {
      return 'This passkey is already on your account.';
    }
    if (
      failure.name === 'NotSupportedError' ||
      failure.name === 'SecurityError'
    ) {
      return 'Passkeys are unavailable here. Try another browser or sign-in method.';
    }
  }
  if (failure instanceof TypeError)
    return 'Couldn’t connect. Try again in a moment.';
  return messageFor('internal_error');
}
