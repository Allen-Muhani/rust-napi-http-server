// Run with: node --import @oxc-node/core/register examples/server.ts
// Then, e.g.: curl http://127.0.0.1:3000/hello/world
import { Application, type HttpRequest, type HttpResponse } from '../index.js'

const app = new Application()

app.get('/hello/:name', (req: HttpRequest, res: HttpResponse): HttpResponse => {
  res.statusCode = 200
  res.headers = [{ name: 'Content-Type', value: 'text/plain' }]
  res.body = `Hello, ${req.pathParams.name}!`
  return res
})

app.post('/echo', (req: HttpRequest, res: HttpResponse): HttpResponse => {
  res.statusCode = 200
  res.headers = [{ name: 'Content-Type', value: 'text/plain' }]
  res.body = req.body
  return res
})

const port = 3000
app.start(port)
console.log(`listening on http://127.0.0.1:${port}`)
