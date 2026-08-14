import { ref } from 'vue'

/**
 * WebAuthn (Passkey / fingerprint) composable.
 *
 * Registration:   POST /api/v1/auth/webauthn/register/begin|finish
 * Authentication: POST /api/v1/auth/webauthn/login/begin|finish
 *
 * The server wraps every response in { code, msg, data }.  Both the
 * challenge and the publicKey options are nested inside `.data.publicKey`
 * (the standard webauthn-rs JSON format), so we unwrap accordingly.
 *
 * Error codes thrown (never localised strings — Login.vue maps them via
 * toUserMessage()):
 *   WEBAUTHN_NOT_SUPPORTED  — browser has no PublicKeyCredential
 *   REGISTER_INIT_FAILED    — begin endpoint returned non-2xx
 *   REGISTER_FAILED         — finish endpoint returned non-2xx
 *   WEBAUTHN_INIT_FAILED    — login/begin returned non-2xx
 *   WEBAUTHN_FAILED         — login/finish returned non-2xx
 *   UNEXPECTED_RESPONSE     — server body could not be unwrapped
 */
export function useWebAuthn() {
  const supported = ref(
    typeof window !== 'undefined' && !!window.PublicKeyCredential,
  )
  const registering = ref(false)
  const authenticating = ref(false)

  // ── base64url helpers ──────────────────────────────────────────────────────

  function b64ToBuffer(b64: string): ArrayBuffer {
    const bin = atob(b64.replace(/-/g, '+').replace(/_/g, '/'))
    const buf = new Uint8Array(bin.length)
    for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i)
    return buf.buffer
  }

  function bufferToB64(buf: ArrayBuffer): string {
    return btoa(String.fromCharCode(...new Uint8Array(buf)))
      .replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
  }

  /** Recursively decode all base64url `id` / `challenge` / `user.id` fields. */
  function decodeOptions(opts: Record<string, unknown>): void {
    if (typeof opts.challenge === 'string') opts.challenge = b64ToBuffer(opts.challenge)
    if (opts.user && typeof (opts.user as Record<string, unknown>).id === 'string') {
      ;(opts.user as Record<string, unknown>).id = b64ToBuffer(
        (opts.user as Record<string, unknown>).id as string,
      )
    }
    for (const key of ['excludeCredentials', 'allowCredentials'] as const) {
      const arr = opts[key]
      if (Array.isArray(arr)) {
        opts[key] = arr.map((c: { id: string; type: string }) => ({ ...c, id: b64ToBuffer(c.id) }))
      }
    }
  }

  /** Unwrap `{ code, msg, data: { publicKey: ... } }` → publicKey options object. */
  function unwrapPublicKey(body: unknown): Record<string, unknown> {
    if (body && typeof body === 'object') {
      const b = body as Record<string, unknown>
      if (b.data && typeof b.data === 'object') {
        const d = b.data as Record<string, unknown>
        if (d.publicKey && typeof d.publicKey === 'object') {
          return d.publicKey as Record<string, unknown>
        }
        return d
      }
      return b
    }
    throw new Error('UNEXPECTED_RESPONSE')
  }

  // ── registration ──────────────────────────────────────────────────────────

  async function register(username: string): Promise<void> {
    if (!supported.value) throw new Error('WEBAUTHN_NOT_SUPPORTED')
    registering.value = true
    try {
      const beginRes = await fetch('/api/v1/auth/webauthn/register/begin', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      })
      if (!beginRes.ok) throw new Error('REGISTER_INIT_FAILED')

      const options = unwrapPublicKey(await beginRes.json())
      decodeOptions(options)

      const credential = await navigator.credentials.create(
        { publicKey: options as PublicKeyCredentialCreationOptions },
      ) as PublicKeyCredential
      const response = credential.response as AuthenticatorAttestationResponse

      const finishRes = await fetch('/api/v1/auth/webauthn/register/finish', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          username,
          credential: {
            id: credential.id,
            rawId: bufferToB64(credential.rawId),
            type: credential.type,
            response: {
              attestationObject: bufferToB64(response.attestationObject),
              clientDataJSON: bufferToB64(response.clientDataJSON),
            },
          },
        }),
      })
      if (!finishRes.ok) throw new Error('REGISTER_FAILED')
    } finally {
      registering.value = false
    }
  }

  // ── authentication ────────────────────────────────────────────────────────

  async function authenticate(): Promise<boolean> {
    if (!supported.value) throw new Error('WEBAUTHN_NOT_SUPPORTED')
    authenticating.value = true
    try {
      const beginRes = await fetch('/api/v1/auth/webauthn/login/begin', {
        method: 'POST',
        credentials: 'include',
      })
      if (!beginRes.ok) throw new Error('WEBAUTHN_INIT_FAILED')

      const options = unwrapPublicKey(await beginRes.json())
      decodeOptions(options)

      const credential = await navigator.credentials.get(
        { publicKey: options as PublicKeyCredentialRequestOptions },
      ) as PublicKeyCredential
      const response = credential.response as AuthenticatorAssertionResponse

      const finishRes = await fetch('/api/v1/auth/webauthn/login/finish', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: credential.id,
          rawId: bufferToB64(credential.rawId),
          type: credential.type,
          response: {
            authenticatorData: bufferToB64(response.authenticatorData),
            clientDataJSON: bufferToB64(response.clientDataJSON),
            signature: bufferToB64(response.signature),
            userHandle: response.userHandle ? bufferToB64(response.userHandle) : null,
          },
        }),
      })
      return finishRes.ok
    } finally {
      authenticating.value = false
    }
  }

  return { supported, registering, authenticating, register, authenticate }
}
