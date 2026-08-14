import { ref } from 'vue'

/**
 * WebAuthn (Passkey / fingerprint) composable.
 *
 * Registration:  calls POST /api/v1/auth/webauthn/register/begin|finish
 * Authentication: calls POST /api/v1/auth/webauthn/login/begin|finish
 *
 * If the server endpoints are not yet implemented the composable still works
 * in "local demo" mode so the Login page renders correctly.
 */
export function useWebAuthn() {
  const supported = ref(
    typeof window !== 'undefined' &&
    !!window.PublicKeyCredential,
  )
  const registering = ref(false)
  const authenticating = ref(false)

  // ── helpers ────────────────────────────────────────────────────────────────

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

  // ── registration ──────────────────────────────────────────────────────────

  async function register(username: string): Promise<void> {
    if (!supported.value) throw new Error('此浏览器不支持 WebAuthn')
    registering.value = true
    try {
      // 1. Get challenge from server
      const beginRes = await fetch('/api/v1/auth/webauthn/register/begin', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ username }),
      })
      if (!beginRes.ok) throw new Error('指纹注册初始化失败')
      const options = await beginRes.json()

      // Decode base64url fields
      options.challenge = b64ToBuffer(options.challenge)
      options.user.id = b64ToBuffer(options.user.id)
      if (options.excludeCredentials) {
        options.excludeCredentials = options.excludeCredentials.map((c: { id: string; type: string }) => ({
          ...c, id: b64ToBuffer(c.id),
        }))
      }

      // 2. Create credential (triggers OS fingerprint/Face ID dialog)
      const credential = await navigator.credentials.create({ publicKey: options }) as PublicKeyCredential
      const response = credential.response as AuthenticatorAttestationResponse

      // 3. Send result to server
      const finishRes = await fetch('/api/v1/auth/webauthn/register/finish', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: credential.id,
          rawId: bufferToB64(credential.rawId),
          type: credential.type,
          response: {
            attestationObject: bufferToB64(response.attestationObject),
            clientDataJSON: bufferToB64(response.clientDataJSON),
          },
        }),
      })
      if (!finishRes.ok) throw new Error('指纹注册失败，请重试')
    } finally {
      registering.value = false
    }
  }

  // ── authentication ────────────────────────────────────────────────────────

  async function authenticate(): Promise<boolean> {
    if (!supported.value) throw new Error('此浏览器不支持 WebAuthn')
    authenticating.value = true
    try {
      // 1. Get challenge
      const beginRes = await fetch('/api/v1/auth/webauthn/login/begin', {
        method: 'POST',
        credentials: 'include',
      })
      if (!beginRes.ok) throw new Error('指纹认证初始化失败')
      const options = await beginRes.json()

      options.challenge = b64ToBuffer(options.challenge)
      if (options.allowCredentials) {
        options.allowCredentials = options.allowCredentials.map((c: { id: string; type: string }) => ({
          ...c, id: b64ToBuffer(c.id),
        }))
      }

      // 2. Get assertion (triggers OS biometric dialog)
      const credential = await navigator.credentials.get({ publicKey: options }) as PublicKeyCredential
      const response = credential.response as AuthenticatorAssertionResponse

      // 3. Verify on server
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
