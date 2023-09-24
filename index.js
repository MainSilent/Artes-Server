const { execSync } = require('child_process')
const ffi = require('ffi-napi')
const tls = require('tls')
const fs = require('fs')
require('dotenv').config()


const PORT = 443
let isAuth = false

const tls_options = {
  key: fs.readFileSync('cert/server.key'),
  cert: fs.readFileSync('cert/server.crt')
}

execSync(`./setup_interface.sh artes "10.31.0.1"`)

const tun = ffi.Library('./tun', {
  'getfd': [ 'int', [] ]
})
const fd = tun.getfd()

const tunRead = fs.createReadStream(null, { fd })
const tunWrite = fs.createWriteStream(null, { fd })


const server = tls.createServer(tls_options, client => {
  console.log('Client connected via TLS')

  client.write(Buffer.from([1]))

  client.on('data', data => {
    if (isAuth)
      tunWrite.write(data)
    else {
      const [ username, password ] = data.toString().split('\0')
      
      if (username == process.env.USERNAME && password == process.env.PASSWORD) {
        isAuth = true
        client.write(Buffer.from([100, 10, 31, 0, 2]))
      } else {
        isAuth = false
        client.write(Buffer.from([99]))
        client.end()
      }
    }
  })

  client.on('end', () => {
    isAuth = false
    console.log('Client disconnected')
  })

  tunRead.on('data', packet => {
    if (isAuth)
      client.write(packet)
  })
})


server.listen(PORT, () => {
  console.log(`Server is listening on port ${PORT}`)
})