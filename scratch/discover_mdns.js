const mdns = require('multicast-dns')()

console.log('Listening for mDNS services (_p2p._udp.local)...')

mdns.on('response', (response) => {
  response.answers.forEach((answer) => {
    if (answer.name.includes('_p2p._udp.local')) {
      console.log('Found answer:', answer.name, answer.data)
    }
  })
  response.additionals.forEach((additional) => {
    if (additional.name.includes('_p2p._udp.local')) {
      console.log('Found additional:', additional.name, additional.data)
    }
  })
})

// Query for the service
mdns.query({
  questions: [{
    name: '_p2p._udp.local',
    type: 'PTR'
  }]
})

setTimeout(() => {
  console.log('Finished listening.')
  process.exit(0)
}, 10000)
