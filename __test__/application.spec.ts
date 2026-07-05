import { createServer, type AddressInfo } from 'node:net'
import test from 'ava'

import { Application } from '../index'

/** Finds a free TCP port by briefly binding to port 0 and reading it back. */
async function getAvailablePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer()
    server.unref()
    server.on('error', reject)
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address() as AddressInfo
      server.close(() => resolve(port))
    })
  })
}

/**
 * `Application.start` binds the listener on a background thread and returns
 * immediately, so the socket may not be accepting connections yet. Polls
 * until a request succeeds (any HTTP status counts as "up").
 */
async function waitForServer(port: number, attempts = 40): Promise<void> {
  for (let i = 0; i < attempts; i++) {
    try {
      await fetch(`http://127.0.0.1:${port}/`)
      return
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 25))
    }
  }
  throw new Error(`server on port ${port} never came up`)
}

test('routes a GET request with a path parameter to its handler', async (t) => {
  const app = new Application()
  const port = await getAvailablePort()

  // Routes must be registered before `start()`: it moves the route table
  // into the background listener thread.
  app.get('/hello/:name', (req, res) => {
    res.statusCode = 200
    res.headers = [{ name: 'Content-Type', value: 'text/plain' }]
    res.body = `Hello, ${req.pathParams.name}!`
    return res
  })
  app.start(port)
  await waitForServer(port)

  const response = await fetch(`http://127.0.0.1:${port}/hello/world`)

  t.is(response.status, 200)
  t.is(response.headers.get('content-type'), 'text/plain')
  t.is(await response.text(), 'Hello, world!')
})

test('returns 404 when no route matches', async (t) => {
  const app = new Application()
  const port = await getAvailablePort()
  app.start(port)
  await waitForServer(port)

  const response = await fetch(`http://127.0.0.1:${port}/nope`)

  t.is(response.status, 404)
})

test('only matches the registered method', async (t) => {
  const app = new Application()
  const port = await getAvailablePort()

  app.post('/items', (req, res) => {
    res.statusCode = 201
    res.body = req.body
    return res
  })
  app.start(port)
  await waitForServer(port)

  const getResponse = await fetch(`http://127.0.0.1:${port}/items`)
  t.is(getResponse.status, 404)

  const postResponse = await fetch(`http://127.0.0.1:${port}/items`, {
    method: 'POST',
    body: 'created!',
  })
  t.is(postResponse.status, 201)
  t.is(await postResponse.text(), 'created!')
})
