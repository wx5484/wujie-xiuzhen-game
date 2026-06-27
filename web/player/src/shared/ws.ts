import type { WsEnvelope } from './types'

export class GameSocket {
  private socket: WebSocket | null = null
  private seq = 1
  private listeners = new Set<(event: WsEnvelope) => void>()
  private queue: string[] = []
  private opened = false

  connect(onOpen?: () => void, onClose?: () => void) {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    this.socket = new WebSocket(`${proto}//${window.location.host}/ws`)
    this.socket.addEventListener('open', () => {
      this.opened = true
      onOpen?.()
      this.flush()
    })
    this.socket.addEventListener('message', (message) => {
      const parsed = JSON.parse(message.data) as WsEnvelope
      this.listeners.forEach((listener) => listener(parsed))
    })
    this.socket.addEventListener('close', () => {
      this.opened = false
      onClose?.()
    })
    this.socket.addEventListener('error', () => {
      this.opened = false
      onClose?.()
    })
  }

  close() {
    this.opened = false
    this.queue = []
    this.socket?.close()
    this.socket = null
  }

  on(listener: (event: WsEnvelope) => void) {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  send(type: string, payload: unknown = {}) {
    const envelope = { type, seq: this.seq++, payload }
    const serialized = JSON.stringify(envelope)
    if (this.opened && this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(serialized)
      return
    }
    this.queue.push(serialized)
  }

  private flush() {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) return
    while (this.queue.length) {
      const next = this.queue.shift()
      if (next) this.socket.send(next)
    }
  }
}
