#!/usr/bin/env node

const baseUrl = process.env.BASE_URL || 'http://127.0.0.1:3000'
const token = process.env.TOKEN
const characterId = Number(process.env.CHARACTER_ID)

if (!token || !Number.isFinite(characterId) || characterId <= 0) {
  console.error('TOKEN and CHARACTER_ID are required')
  process.exit(2)
}

if (typeof WebSocket === 'undefined') {
  console.error('Node.js WebSocket global is required')
  process.exit(2)
}

const url = new URL(baseUrl)
url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
url.pathname = '/ws'
url.search = ''

let seq = 1
let authed = false
let sawRoom = false
let sawCombatOrSafeRoom = false
let lastRoom = null
const pending = []

function send(socket, type, payload = {}) {
  socket.send(JSON.stringify({ type, seq: seq++, payload }))
}

function waitFor(predicate, timeoutMs, label) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      const index = pending.indexOf(handler)
      if (index >= 0) pending.splice(index, 1)
      reject(new Error(`timeout waiting for ${label}`))
    }, timeoutMs)
    function handler(event) {
      if (!predicate(event)) return
      clearTimeout(timer)
      const index = pending.indexOf(handler)
      if (index >= 0) pending.splice(index, 1)
      resolve(event)
    }
    pending.push(handler)
  })
}

const socket = new WebSocket(url)

const done = new Promise((resolve, reject) => {
  const hardTimeout = setTimeout(() => reject(new Error('websocket smoke timed out')), 15000)

  socket.addEventListener('open', async () => {
    try {
      send(socket, 'auth', { token, character_id: characterId, device: 'smoke' })
      await waitFor((event) => event.type === 'auth_ok', 5000, 'auth_ok')
      authed = true

      send(socket, 'state_request')
      await waitFor((event) => event.type === 'room_state', 5000, 'initial room_state')
      sawRoom = true

      const exits = lastRoom?.room?.exits || {}
      if (exits.north) {
        send(socket, 'cmd', { command: 'move', args: { direction: 'north' } })
        await waitFor((event) => event.type === 'room_state', 5000, 'moved room_state')
      }

      if (Array.isArray(lastRoom?.mobs) && lastRoom.mobs.length > 0) {
        send(socket, 'cmd', { command: 'attack', args: { target_id: 0 } })
        await waitFor((event) => event.type === 'combat_log' || event.type === 'error', 5000, 'attack result')
        sawCombatOrSafeRoom = true
      } else {
        sawCombatOrSafeRoom = true
      }

      clearTimeout(hardTimeout)
      socket.close()
      resolve()
    } catch (error) {
      clearTimeout(hardTimeout)
      socket.close()
      reject(error)
    }
  })

  socket.addEventListener('message', (message) => {
    let event
    try {
      event = JSON.parse(message.data)
    } catch {
      return
    }
    if (event.type === 'room_state') {
      lastRoom = event.payload
    }
    if (event.type === 'error' && !authed) {
      reject(new Error(event.payload?.message || 'websocket auth failed'))
      return
    }
    for (const handler of [...pending]) handler(event)
  })

  socket.addEventListener('error', () => {
    clearTimeout(hardTimeout)
    reject(new Error('websocket connection error'))
  })
})

await done

if (!authed || !sawRoom || !sawCombatOrSafeRoom) {
  console.error('websocket smoke did not complete required checks')
  process.exit(1)
}

console.log('websocket smoke ok')
