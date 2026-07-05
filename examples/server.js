// Run with: node examples/server.js
// Then, e.g.: curl http://127.0.0.1:3000/hello/world
const { Application } = require('../index.js')

const app = new Application()

app.get('/hello/:name', (req, res) => {
  res.statusCode = 200
  res.headers = [{ name: 'Content-Type', value: 'text/plain' }]
  res.body = `Hello, ${req.pathParams.name}!`
  return res
})

app.post('/echo', (req, res) => {
  res.statusCode = 200
  
  res.headers = [{ name: 'Content-Type', value: 'text/plain' }]
  res.body = `You said: ${req.body}`
  return res
})

const port = 3000
app.start(port)
